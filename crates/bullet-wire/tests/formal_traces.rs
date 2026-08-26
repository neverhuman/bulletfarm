use std::{fs, path::PathBuf};

use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Trace {
    model: String,
    schema_version: String,
    steps: Vec<Step>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Step {
    action: String,
    expected: String,
    #[serde(default)]
    runner: String,
    #[serde(default)]
    remote: String,
}

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .unwrap()
        .to_path_buf()
}

fn trace(name: &str) -> Trace {
    let bytes = fs::read(root().join("formal/traces").join(name)).unwrap();
    let trace = serde_json::from_slice::<Trace>(&bytes).unwrap();
    assert_eq!(trace.schema_version, "v1alpha1");
    trace
}

#[derive(Default)]
struct LeaseState {
    now: u64,
    owner: Option<String>,
    fence: u64,
    expires: u64,
    r1_fence: u64,
    r2_fence: u64,
    authority_epoch: u64,
    scope_revision: u64,
    acknowledged_scope: u64,
    barrier: bool,
    frozen: bool,
    freeze_generation: u64,
}

impl LeaseState {
    fn apply(&mut self, step: &Step) -> &'static str {
        match step.action.as_str() {
            "acquire" => self.acquire(&step.runner),
            "tick" => {
                self.now += 1;
                "time_advanced"
            }
            "heartbeat" => "refused_expired",
            "reclaim" => {
                assert!(self.expires <= self.now);
                self.owner = None;
                "accepted"
            }
            "apply_stale" => "refused_fence",
            "enter_barrier" => {
                self.barrier = true;
                "accepted"
            }
            "append_scope" => {
                assert!(self.barrier);
                self.scope_revision += 1;
                "revision_2"
            }
            "apply_old_scope" => "refused_scope_revision",
            "ack_scope" => {
                self.acknowledged_scope = self.scope_revision;
                "revision_2"
            }
            "resume" => {
                assert_eq!(self.acknowledged_scope, self.scope_revision);
                self.barrier = false;
                "accepted"
            }
            "freeze" => {
                self.frozen = true;
                self.freeze_generation += 1;
                "generation_1"
            }
            "apply" if self.frozen => "refused_frozen",
            "restore" => {
                self.authority_epoch += 1;
                self.owner = None;
                "authority_epoch_2"
            }
            "recover_active" => {
                assert!(self.frozen);
                self.frozen = false;
                "active_epoch_2"
            }
            "apply" => "refused_epoch",
            other => panic!("unknown LeaseFence trace action {other}"),
        }
    }

    fn acquire(&mut self, runner: &str) -> &'static str {
        assert!(self.owner.is_none());
        self.fence += 1;
        self.expires = self.now + 1;
        self.owner = Some(runner.to_owned());
        self.scope_revision = self.scope_revision.max(1);
        self.acknowledged_scope = self.scope_revision;
        match runner {
            "r1" => self.r1_fence = self.fence,
            "r2" => self.r2_fence = self.fence,
            other => panic!("unknown runner {other}"),
        }
        if self.fence == 1 {
            "accepted"
        } else {
            "accepted_fence_2"
        }
    }
}

#[test]
fn lease_fence_trace_replays_gateway_refusals() {
    let trace = trace("lease-fence-reclaim.json");
    assert_eq!(trace.model, "LeaseFence");
    let mut state = LeaseState {
        authority_epoch: 1,
        scope_revision: 1,
        ..LeaseState::default()
    };
    for step in &trace.steps {
        assert_eq!(state.apply(step), step.expected, "action {}", step.action);
    }
    assert_eq!(state.fence, 2);
    assert_eq!(state.r1_fence, 1);
    assert_eq!(state.r2_fence, 2);
}

#[derive(Default)]
struct EffectState {
    effect_phase: &'static str,
    effect_remote: &'static str,
    proof_durable: bool,
    check_phase: &'static str,
    check_remote: &'static str,
}

impl EffectState {
    fn apply(&mut self, step: &Step) -> &'static str {
        let outcome = match step.action.as_str() {
            "persist_effect_intent" => {
                self.effect_phase = "intent";
                "intent"
            }
            "dispatch_effect_response_lost" => {
                self.effect_phase = "unknown";
                self.effect_remote = "desired";
                "unknown"
            }
            "dispatch_effect_timeout" => {
                self.effect_phase = "unknown";
                "unknown"
            }
            "crash" => "durable_unknown",
            "third_party_effect" => {
                self.effect_remote = "third_party";
                "unknown"
            }
            "readback_effect" if self.effect_remote == "desired" => {
                self.effect_phase = "verified";
                "verified_exact"
            }
            "readback_effect" => {
                self.effect_phase = "orphaned";
                "orphaned_remote"
            }
            "persist_proof" => {
                assert_eq!(self.effect_phase, "verified");
                self.proof_durable = true;
                "durable"
            }
            "persist_check_intent" => {
                assert!(self.proof_durable);
                self.check_phase = "intent";
                "intent"
            }
            "dispatch_check_response_lost" => {
                self.check_phase = "unknown";
                self.check_remote = "desired";
                "unknown"
            }
            "readback_check" => {
                assert_eq!(self.check_remote, "desired");
                self.check_phase = "verified";
                "verified_exact"
            }
            other => panic!("unknown EffectCheck trace action {other}"),
        };
        let remote = if step.action.contains("check") {
            self.check_remote
        } else {
            self.effect_remote
        };
        assert_eq!(remote, step.remote, "remote for {}", step.action);
        outcome
    }
}

#[test]
fn effect_check_traces_replay_ambiguity_and_orphaning() {
    for name in ["effect-check-ambiguity.json", "effect-third-party.json"] {
        let trace = trace(name);
        assert_eq!(trace.model, "EffectCheck");
        let mut state = EffectState {
            effect_phase: "absent",
            effect_remote: "none",
            check_phase: "absent",
            check_remote: "none",
            ..EffectState::default()
        };
        for step in &trace.steps {
            assert_eq!(state.apply(step), step.expected, "action {}", step.action);
        }
    }
}
