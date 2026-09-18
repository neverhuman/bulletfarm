use serde::Deserialize;
use serde_json::json;
mod hostile;

use super::*;
use crate::*;

type MutationCase<T> = (fn(&mut T), &'static str);

const BASE_FIXTURE: &str = concat!(
    r#"{"passport":{"schema_version":1,"provider":"claude","protocol":"claude_stream_json","version":"2.1.251","deployment_root":"/usr/lib/bullet/providers/claude/2.1.251","entrypoint":"bin/claude","execution":{"kind":"native","loader":{"kind":"static"}},"files":[{"path":"bin/claude","role":"entrypoint","mode":365,"size":1,"blake3":"1111111111111111111111111111111111111111111111111111111111111111"}],"aggregate_file_count":1,"aggregate_size_bytes":1},"enrollment":{"schema_version":"v1alpha1","issuer":"operator.example","key_id":"provider-enrollment-alpha","signing_purpose":"provider-enrollment-signing","claims_domain":"provider.enrollment-claims.v2","provider":"claude","protocol":"claude_stream_json","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","service_identity_id":"pri_0303030303030303030303030303030303030303030303030303030303030303","credential_projection_profile_id":"cpp_0404040404040404040404040404040404040404040404040404040404040404","runtime_version":"2.1.251","enrollment_generation":2,"activates_at_unix_ms":1000,"expires_at_unix_ms":5000,"revoked_at_unix_ms":null,"egress_policy_digest":"0505050505050505050505050505050505050505050505050505050505050505","tool_policy_digest":"0606060606060606060606060606060606060606060606060606060606060606","budget_policy_digest":"0707070707070707070707070707070707070707070707070707070707070707","endpoint_observation_digest":"bdc3fbc09c5d29de0a65ecfac5268ed6c78b6d1f55828d1027feb88986eae9b3","version_observation_digest":"ea68136c1ee268deec26242eb290f754804239638756e40a9e68700c212f54f3","profile_observation_digest":"0116aa96c7d129e9a351d09c0fb317852fe69bbb230fc795833d925f28777900","policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2},"#,
    r#""intent":{"schema_version":"v1alpha1","request_digest":"1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f","subject":{"execution":{"command_id":"cmd_2020202020202020202020202020202020202020202020202020202020202020","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","mission_id":"mis_2121212121212121212121212121212121212121212121212121212121212121","repository_id":"rep_1212121212121212121212121212121212121212121212121212121212121212","graph_revision_id":"grf_2222222222222222222222222222222222222222222222222222222222222222","work_package_id":"wpk_2323232323232323232323232323232323232323232323232323232323232323","variant_id":"var_2424242424242424242424242424242424242424242424242424242424242424","attempt_id":"atm_1414141414141414141414141414141414141414141414141414141414141414","attempt_fence":3,"runner_id":"run_2525252525252525252525252525252525252525252525252525252525252525","runner_epoch":4,"authority_epoch":5,"freeze_generation":6},"provider":{"provider":"claude","protocol":"claude_stream_json","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_enrollment_id":"pen_951b5254b7d1c170f15dbbd9dd09ca484677d896846a1730fd44b49dc19beae7","credential_projection_id":"pcp_2626262626262626262626262626262626262626262626262626262626262626"},"repository":{"context_snapshot_id":"rcs_91a9e83a09aa6dad3fe9ba5701aa1e7090330b8af07fad1b62e37a1fa3a7b2c7","head_oid":"sha256:1616161616161616161616161616161616161616161616161616161616161616","tree_oid":"sha256:1717171717171717171717171717171717171717171717171717171717171717","checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818"},"gate_ids":["gat_1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d","gat_1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e"],"prompt_digest":"2727272727272727272727272727272727272727272727272727272727272727","policy":{"policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2,"dogfood_binding_digest":"2828282828282828282828282828282828282828282828282828282828282828","tool_policy_digest":"0606060606060606060606060606060606060606060606060606060606060606","egress_policy_digest":"0505050505050505050505050505050505050505050505050505050505050505","containment_policy_digest":"2929292929292929292929292929292929292929292929292929292929292929"},"budget_reservation_id":"dbr_2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a","deadline_unix_ms":3000,"output_schema_digest":"2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"}},"grant":{"schema_version":"v1alpha1","audience":"dogfood-runner","operation":"read-only-propose","issuer":"kernel.example","key_id":"dogfood-launch-alpha","signing_purpose":"dogfood-launch-signing","claims_domain":"authority.dogfood-launch-grant-claims.v1alpha1","issued_at_unix_ms":1900,"not_before_unix_ms":2000,"expires_at_unix_ms":2800,"grant_nonce":"2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c","request_digest":"1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f","intent_id":"dfi_c84d3bc08ac144e2b99c7537c332fa4d286c0e10719e374f1c428fe305fd5b23","subject":{"execution":{"command_id":"cmd_2020202020202020202020202020202020202020202020202020202020202020","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","mission_id":"mis_2121212121212121212121212121212121212121212121212121212121212121","repository_id":"rep_1212121212121212121212121212121212121212121212121212121212121212","graph_revision_id":"grf_2222222222222222222222222222222222222222222222222222222222222222","work_package_id":"wpk_2323232323232323232323232323232323232323232323232323232323232323","variant_id":"var_2424242424242424242424242424242424242424242424242424242424242424","attempt_id":"atm_1414141414141414141414141414141414141414141414141414141414141414","attempt_fence":3,"runner_id":"run_2525252525252525252525252525252525252525252525252525252525252525","runner_epoch":4,"authority_epoch":5,"freeze_generation":6},"provider":{"provider":"claude","protocol":"claude_stream_json","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_enrollment_id":"pen_951b5254b7d1c170f15dbbd9dd09ca484677d896846a1730fd44b49dc19beae7","credential_projection_id":"pcp_2626262626262626262626262626262626262626262626262626262626262626"},"repository":{"context_snapshot_id":"rcs_91a9e83a09aa6dad3fe9ba5701aa1e7090330b8af07fad1b62e37a1fa3a7b2c7","head_oid":"sha256:1616161616161616161616161616161616161616161616161616161616161616","tree_oid":"sha256:1717171717171717171717171717171717171717171717171717171717171717","checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818"},"gate_ids":["gat_1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d","gat_1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e"],"prompt_digest":"2727272727272727272727272727272727272727272727272727272727272727","policy":{"policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2,"dogfood_binding_digest":"2828282828282828282828282828282828282828282828282828282828282828","tool_policy_digest":"0606060606060606060606060606060606060606060606060606060606060606","egress_policy_digest":"0505050505050505050505050505050505050505050505050505050505050505","containment_policy_digest":"2929292929292929292929292929292929292929292929292929292929292929"},"budget_reservation_id":"dbr_2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a","deadline_unix_ms":3000,"output_schema_digest":"2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"}},"#,
    r#""projection":{"schema_version":"v1alpha1","projection_instance_id":"pcp_2626262626262626262626262626262626262626262626262626262626262626","credential_projection_profile_id":"cpp_0404040404040404040404040404040404040404040404040404040404040404","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","provider":"claude","service_identity_id":"pri_0303030303030303030303030303030303030303030303030303030303030303","activates_at_unix_ms":2000,"expires_at_unix_ms":2800,"target_policy_digest":"2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d","secret_commitment_digest":"2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e"},"reservation":{"schema_version":"v1alpha1","reservation_id":"dbr_2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","provider":"claude","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","provider_enrollment_id":"pen_951b5254b7d1c170f15dbbd9dd09ca484677d896846a1730fd44b49dc19beae7","budget_policy_digest":"0707070707070707070707070707070707070707070707070707070707070707","reserved_at_unix_ms":2000,"consume_before_unix_ms":2800,"reserved_cost_micro_usd":1000,"reserved_invocations":1,"reserved_wall_time_ms":500,"reserved_concurrency":1},"#,
    r#""context":{"schema_version":"v1alpha1","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","repository_id":"rep_1212121212121212121212121212121212121212121212121212121212121212","source_descriptor_id":"src_1313131313131313131313131313131313131313131313131313131313131313","attempt_id":"atm_1414141414141414141414141414141414141414141414141414141414141414","attempt_fence":3,"owner_principal_id":"pri_1515151515151515151515151515151515151515151515151515151515151515","head_oid":"sha256:1616161616161616161616161616161616161616161616161616161616161616","tree_oid":"sha256:1717171717171717171717171717171717171717171717171717171717171717","checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818","checkpoint_digest":"1919191919191919191919191919191919191919191919191919191919191919","scope_grant_digest":"1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a","visible_scopes":["src"],"files":[{"path":"src/lib.rs","preimage_digest":"1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b","size_bytes":10,"executable":false}],"aggregate_file_count":1,"aggregate_size_bytes":10,"visible_manifest_digest":"ddf77c46f0528a03057b96cb51ee08c492a87caf169a00c6463fe76e656ba232","prepared_at_unix_ms":2000},"post":{"schema_version":"v1alpha1","context_snapshot_id":"rcs_91a9e83a09aa6dad3fe9ba5701aa1e7090330b8af07fad1b62e37a1fa3a7b2c7","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","observer_principal_id":"pri_2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f","observed_at_unix_ms":2200,"observed_owner_principal_id":"pri_1515151515151515151515151515151515151515151515151515151515151515","observed_head_oid":"sha256:1616161616161616161616161616161616161616161616161616161616161616","observed_tree_oid":"sha256:1717171717171717171717171717171717171717171717171717171717171717","observed_checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818","observed_checkpoint_digest":"1919191919191919191919191919191919191919191919191919191919191919","observed_visible_manifest_digest":"ddf77c46f0528a03057b96cb51ee08c492a87caf169a00c6463fe76e656ba232"},"#,
    r#""probe":{"schema_version":"v1alpha1","subject":{"provider":"claude","protocol":"claude_stream_json","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","service_identity_id":"pri_0303030303030303030303030303030303030303030303030303030303030303","credential_projection_profile_id":"cpp_0404040404040404040404040404040404040404040404040404040404040404","policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2},"probe_grant_digest":"0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c","containment_receipt_digest":"0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d","protocol_transcript_digest":"0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e","observed_at_unix_ms":900},"endpoint":{"schema_version":"v1alpha1","subject":{"provider":"claude","protocol":"claude_stream_json","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","service_identity_id":"pri_0303030303030303030303030303030303030303030303030303030303030303","credential_projection_profile_id":"cpp_0404040404040404040404040404040404040404040404040404040404040404","policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2},"probe_observation_digest":"9ee793b9cb4124b834d783d3053986f5c90da8aaf093eb46c899aa398f5248ea","entrypoint_blake3":"1111111111111111111111111111111111111111111111111111111111111111"},"#,
    r#""version":{"schema_version":"v1alpha1","subject":{"provider":"claude","protocol":"claude_stream_json","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","service_identity_id":"pri_0303030303030303030303030303030303030303030303030303030303030303","credential_projection_profile_id":"cpp_0404040404040404040404040404040404040404040404040404040404040404","policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2},"probe_observation_digest":"9ee793b9cb4124b834d783d3053986f5c90da8aaf093eb46c899aa398f5248ea","runtime_version":"2.1.251","native_version_artifact_digest":"0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f"},"profile":{"schema_version":"v1alpha1","subject":{"provider":"claude","protocol":"claude_stream_json","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","service_identity_id":"pri_0303030303030303030303030303030303030303030303030303030303030303","credential_projection_profile_id":"cpp_0404040404040404040404040404040404040404040404040404040404040404","policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2},"probe_observation_digest":"9ee793b9cb4124b834d783d3053986f5c90da8aaf093eb46c899aa398f5248ea","effective_identity_artifact_digest":"1010101010101010101010101010101010101010101010101010101010101010"},"#,
    r#""proposal":{"schema_version":1,"proposal_id":"cnt_3030303030303030303030303030303030303030303030303030303030303030","producing_attempt_id":"atm_1414141414141414141414141414141414141414141414141414141414141414","base_checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818","base_checkpoint_digest":"1919191919191919191919191919191919191919191919191919191919191919","operations":[{"path":"src/file-000.rs","preimage":{"kind":"digest","digest":"3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f"},"mutation":{"kind":"write","content_utf8":"fn main() {}\n"}}],"gate_ids":["gat_1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d","gat_1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e"]},"run":{"schema_version":"v1alpha1","subject":{"execution":{"command_id":"cmd_2020202020202020202020202020202020202020202020202020202020202020","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","mission_id":"mis_2121212121212121212121212121212121212121212121212121212121212121","repository_id":"rep_1212121212121212121212121212121212121212121212121212121212121212","graph_revision_id":"grf_2222222222222222222222222222222222222222222222222222222222222222","work_package_id":"wpk_2323232323232323232323232323232323232323232323232323232323232323","variant_id":"var_2424242424242424242424242424242424242424242424242424242424242424","attempt_id":"atm_1414141414141414141414141414141414141414141414141414141414141414","attempt_fence":3,"runner_id":"run_2525252525252525252525252525252525252525252525252525252525252525","runner_epoch":4,"authority_epoch":5,"freeze_generation":6},"provider":{"provider":"claude","protocol":"claude_stream_json","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_enrollment_id":"pen_951b5254b7d1c170f15dbbd9dd09ca484677d896846a1730fd44b49dc19beae7","credential_projection_id":"pcp_2626262626262626262626262626262626262626262626262626262626262626"},"repository":{"context_snapshot_id":"rcs_91a9e83a09aa6dad3fe9ba5701aa1e7090330b8af07fad1b62e37a1fa3a7b2c7","head_oid":"sha256:1616161616161616161616161616161616161616161616161616161616161616","tree_oid":"sha256:1717171717171717171717171717171717171717171717171717171717171717","checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818"},"gate_ids":["gat_1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d","gat_1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e"],"prompt_digest":"2727272727272727272727272727272727272727272727272727272727272727","policy":{"policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2,"dogfood_binding_digest":"2828282828282828282828282828282828282828282828282828282828282828","tool_policy_digest":"0606060606060606060606060606060606060606060606060606060606060606","egress_policy_digest":"0505050505050505050505050505050505050505050505050505050505050505","containment_policy_digest":"2929292929292929292929292929292929292929292929292929292929292929"},"budget_reservation_id":"dbr_2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a","deadline_unix_ms":3000,"output_schema_digest":"2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"},"intent_id":"dfi_c84d3bc08ac144e2b99c7537c332fa4d286c0e10719e374f1c428fe305fd5b23","launch_grant_id":"dfg_d95e65fb3c7b9ad82d88003b276f7fbbbb10b51036d246a42bf742a8787b3084","credential_projection_digest":"add2d0ddbd69262533f950dc6374b7f28917e39d8b7aa4ff64caf5df3d5fb9dd","budget_settlement":{"schema_version":"v1alpha1","reservation_id":"dbr_2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a","reservation_digest":"fed06f921628d0899f70cb12edf635da4dc9afe5a908192dd3ec6a6d98dd6814","settled_at_unix_ms":2300,"cost_micro_usd":{"knowledge":"known","used":100,"released":900,"overrun":0},"invocations":{"knowledge":"known","used":1,"released":0,"overrun":0},"wall_time_ms":{"knowledge":"known","used":100,"released":400,"overrun":0},"concurrency":{"knowledge":"known","used":1,"released":0,"overrun":0}},"repository_context_post_observation_digest":"7727bafb375edae8c83a65e607c126499a82b77c049f242e0dcfea942470b15e","provider_probe_observation_digest":"9ee793b9cb4124b834d783d3053986f5c90da8aaf093eb46c899aa398f5248ea","attestor_principal_id":"pri_3131313131313131313131313131313131313131313131313131313131313131","process":{"state":{"kind":"exited","code":0},"started_at_unix_ms":2000,"ended_at_unix_ms":2200,"observation_digest":"3232323232323232323232323232323232323232323232323232323232323232"},"artifacts":{"stdout":{"digest":"3333333333333333333333333333333333333333333333333333333333333333","size_bytes":10},"stderr":{"digest":"3434343434343434343434343434343434343434343434343434343434343434","size_bytes":11},"events":{"digest":"3535353535353535353535353535353535353535353535353535353535353535","size_bytes":12},"proxy":{"digest":"3636363636363636363636363636363636363636363636363636363636363636","size_bytes":13},"containment_receipt_digest":"3737373737373737373737373737373737373737373737373737373737373737","egress_receipt_digest":"3838383838383838383838383838383838383838383838383838383838383838","canary_observation_digest":"3939393939393939393939393939393939393939393939393939393939393939","process_tree_observation_digest":"3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a","artifact_manifest_digest":"3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b","retained_artifacts":[{"digest":"3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c","size_bytes":14},{"digest":"3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d","size_bytes":15}],"retained_artifact_count":2,"retained_artifact_size_bytes":29},"proposal":{"kind":"validated","proposal_id":"cnt_3030303030303030303030303030303030303030303030303030303030303030","proposal_digest":"838c6d578f8ac3fe2a429587b79426400e4697e45c31d1c682ff10bdfd1833d4","artifact":{"digest":"110513d7d9932595447022880ed45e8d1b90a03d939957596fdbb0dcf37d9212","size_bytes":745}},"cleanup":{"kind":"proved_empty","receipt_digest":"3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e","observed_at_unix_ms":2400},"attested_at_unix_ms":2500}}"#,
);

const POLICY: &[u8] = include_bytes!("../../../tests/fixtures/policy-v1alpha2-live-enabled.json");
const ATT: &str = "pri_3131313131313131313131313131313131313131313131313131313131313131";
const LAUNCH: &str = "pri_4141414141414141414141414141414141414141414141414141414141414141";
const ENROLL: &str = "pri_4242424242424242424242424242424242424242424242424242424242424242";
const LIVE: &str = "pri_4343434343434343434343434343434343434343434343434343434343434343";
const RELEASE: &str = "pri_4444444444444444444444444444444444444444444444444444444444444444";
const RUN_KEY: &str = "dogfood-run-1";
const LAUNCH_KEY: &str = "dogfood-launch-1";
const ENROLL_KEY: &str = "provider-enrollment-1";
const LIVE_KEY: &str = "live-1";
const RUN_SECRET: &str = "9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
const RUN_PUBLIC: &str = "d75a980182b10ab7d54bfed3c964073a0ee172f3daa62325af021a68f707511a";
const LAUNCH_PUBLIC: &str = "1eb9dbbbbc047c03fd70604e0071f0987e16b28b757225c11f00415d0e20b1a2";
const ENROLL_PUBLIC: &str = "fc51cd8e6218a1a38da47ed00230f0580816ed13ba3303ac5deb911548908025";
const LIVE_PUBLIC: &str = "278117fc144c72340f67d0f2316e8386ceffbf2b2428c9c51fef7c597f1d426e";
const NOW: u64 = 2_500;
#[rustfmt::skip]
const GOLDEN_CLAUDE_PROPOSAL_READY_RUN: &[u8] = br###"{"artifacts":{"artifact_manifest_digest":"3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b","canary_observation_digest":"3939393939393939393939393939393939393939393939393939393939393939","containment_receipt_digest":"3737373737373737373737373737373737373737373737373737373737373737","egress_receipt_digest":"3838383838383838383838383838383838383838383838383838383838383838","events":{"digest":"3535353535353535353535353535353535353535353535353535353535353535","size_bytes":12},"process_tree_observation_digest":"3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a","proxy":{"digest":"3636363636363636363636363636363636363636363636363636363636363636","size_bytes":13},"retained_artifact_count":2,"retained_artifact_size_bytes":29,"retained_artifacts":[{"digest":"3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c","size_bytes":14},{"digest":"3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d","size_bytes":15}],"stderr":{"digest":"3434343434343434343434343434343434343434343434343434343434343434","size_bytes":11},"stdout":{"digest":"3333333333333333333333333333333333333333333333333333333333333333","size_bytes":10}},"attested_at_unix_ms":2500,"attestor_principal_id":"pri_3131313131313131313131313131313131313131313131313131313131313131","budget_settlement":{"concurrency":{"knowledge":"known","overrun":0,"released":0,"used":1},"cost_micro_usd":{"knowledge":"known","overrun":0,"released":900,"used":100},"invocations":{"knowledge":"known","overrun":0,"released":0,"used":1},"reservation_digest":"7e17ec67344bfff697ce6ad20f573b8a40505de9e6fba2495c85fe1a2fe7e695","reservation_id":"dbr_2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a","schema_version":"v1alpha1","settled_at_unix_ms":2300,"wall_time_ms":{"knowledge":"known","overrun":0,"released":400,"used":100}},"cleanup":{"kind":"proved_empty","observed_at_unix_ms":2400,"receipt_digest":"3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e"},"credential_projection_digest":"add2d0ddbd69262533f950dc6374b7f28917e39d8b7aa4ff64caf5df3d5fb9dd","intent_id":"dfi_0d1caf4c572c760c0ff43635306f4b260ea669d2195d05a0712a7e7e6aa99104","launch_grant_id":"dfg_4c3b0ccdb8d000a9453547e07663875c90909a004a984e1f621e966f180365b4","process":{"ended_at_unix_ms":2200,"observation_digest":"3232323232323232323232323232323232323232323232323232323232323232","started_at_unix_ms":2000,"state":{"code":0,"kind":"exited"}},"proposal":{"artifact":{"digest":"110513d7d9932595447022880ed45e8d1b90a03d939957596fdbb0dcf37d9212","size_bytes":745},"kind":"validated","proposal_digest":"838c6d578f8ac3fe2a429587b79426400e4697e45c31d1c682ff10bdfd1833d4","proposal_id":"cnt_3030303030303030303030303030303030303030303030303030303030303030"},"provider_probe_observation_digest":"1d988e72f666eee6858c128c15b9d2e24606faf842c03089fb856797f28fe13d","repository_context_post_observation_digest":"7727bafb375edae8c83a65e607c126499a82b77c049f242e0dcfea942470b15e","schema_version":"v1alpha1","subject":{"budget_reservation_id":"dbr_2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a","deadline_unix_ms":3000,"execution":{"attempt_fence":3,"attempt_id":"atm_1414141414141414141414141414141414141414141414141414141414141414","authority_epoch":5,"command_id":"cmd_2020202020202020202020202020202020202020202020202020202020202020","freeze_generation":6,"graph_revision_id":"grf_2222222222222222222222222222222222222222222222222222222222222222","mission_id":"mis_2121212121212121212121212121212121212121212121212121212121212121","repository_id":"rep_1212121212121212121212121212121212121212121212121212121212121212","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","runner_epoch":4,"runner_id":"run_2525252525252525252525252525252525252525252525252525252525252525","variant_id":"var_2424242424242424242424242424242424242424242424242424242424242424","work_package_id":"wpk_2323232323232323232323232323232323232323232323232323232323232323"},"gate_ids":["gat_1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d","gat_1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e"],"output_schema_digest":"2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b","policy":{"containment_policy_digest":"2929292929292929292929292929292929292929292929292929292929292929","dogfood_binding_digest":"2828282828282828282828282828282828282828282828282828282828282828","egress_policy_digest":"0505050505050505050505050505050505050505050505050505050505050505","policy_generation":2,"policy_snapshot_digest":"fb23b47a73aad4edab7d5ee01644b8ac24725da82b2e42f75e52911de4e51faa","tool_policy_digest":"0606060606060606060606060606060606060606060606060606060606060606"},"prompt_digest":"2727272727272727272727272727272727272727272727272727272727272727","provider":{"credential_projection_id":"pcp_2626262626262626262626262626262626262626262626262626262626262626","protocol":"claude_stream_json","provider":"claude","provider_enrollment_id":"pen_3d2d4ee36479b1b962b1ed3f22314ff28c7de0cd05ac060b9e0fdc05f0da7508","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e"},"repository":{"checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818","context_snapshot_id":"rcs_91a9e83a09aa6dad3fe9ba5701aa1e7090330b8af07fad1b62e37a1fa3a7b2c7","head_oid":"sha256:1616161616161616161616161616161616161616161616161616161616161616","tree_oid":"sha256:1717171717171717171717171717171717171717171717171717171717171717"}}}"###;
#[rustfmt::skip]
const GOLDEN_CLAUDE_PROPOSAL_READY_TOKEN: &str = r###"v4.public.eyJhcnRpZmFjdHMiOnsiYXJ0aWZhY3RfbWFuaWZlc3RfZGlnZXN0IjoiM2IzYjNiM2IzYjNiM2IzYjNiM2IzYjNiM2IzYjNiM2IzYjNiM2IzYjNiM2IzYjNiM2IzYjNiM2IzYjNiM2IzYiIsImNhbmFyeV9vYnNlcnZhdGlvbl9kaWdlc3QiOiIzOTM5MzkzOTM5MzkzOTM5MzkzOTM5MzkzOTM5MzkzOTM5MzkzOTM5MzkzOTM5MzkzOTM5MzkzOTM5MzkzOTM5IiwiY29udGFpbm1lbnRfcmVjZWlwdF9kaWdlc3QiOiIzNzM3MzczNzM3MzczNzM3MzczNzM3MzczNzM3MzczNzM3MzczNzM3MzczNzM3MzczNzM3MzczNzM3MzczNzM3IiwiZWdyZXNzX3JlY2VpcHRfZGlnZXN0IjoiMzgzODM4MzgzODM4MzgzODM4MzgzODM4MzgzODM4MzgzODM4MzgzODM4MzgzODM4MzgzODM4MzgzODM4MzgzOCIsImV2ZW50cyI6eyJkaWdlc3QiOiIzNTM1MzUzNTM1MzUzNTM1MzUzNTM1MzUzNTM1MzUzNTM1MzUzNTM1MzUzNTM1MzUzNTM1MzUzNTM1MzUzNTM1Iiwic2l6ZV9ieXRlcyI6MTJ9LCJwcm9jZXNzX3RyZWVfb2JzZXJ2YXRpb25fZGlnZXN0IjoiM2EzYTNhM2EzYTNhM2EzYTNhM2EzYTNhM2EzYTNhM2EzYTNhM2EzYTNhM2EzYTNhM2EzYTNhM2EzYTNhM2EzYSIsInByb3h5Ijp7ImRpZ2VzdCI6IjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYiLCJzaXplX2J5dGVzIjoxM30sInJldGFpbmVkX2FydGlmYWN0X2NvdW50IjoyLCJyZXRhaW5lZF9hcnRpZmFjdF9zaXplX2J5dGVzIjoyOSwicmV0YWluZWRfYXJ0aWZhY3RzIjpbeyJkaWdlc3QiOiIzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjM2MzYzNjIiwic2l6ZV9ieXRlcyI6MTR9LHsiZGlnZXN0IjoiM2QzZDNkM2QzZDNkM2QzZDNkM2QzZDNkM2QzZDNkM2QzZDNkM2QzZDNkM2QzZDNkM2QzZDNkM2QzZDNkM2QzZCIsInNpemVfYnl0ZXMiOjE1fV0sInN0ZGVyciI6eyJkaWdlc3QiOiIzNDM0MzQzNDM0MzQzNDM0MzQzNDM0MzQzNDM0MzQzNDM0MzQzNDM0MzQzNDM0MzQzNDM0MzQzNDM0MzQzNDM0Iiwic2l6ZV9ieXRlcyI6MTF9LCJzdGRvdXQiOnsiZGlnZXN0IjoiMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMzMyIsInNpemVfYnl0ZXMiOjEwfX0sImF0dGVzdGVkX2F0X3VuaXhfbXMiOjI1MDAsImF0dGVzdG9yX3ByaW5jaXBhbF9pZCI6InByaV8zMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxIiwiYnVkZ2V0X3NldHRsZW1lbnQiOnsiY29uY3VycmVuY3kiOnsia25vd2xlZGdlIjoia25vd24iLCJvdmVycnVuIjowLCJyZWxlYXNlZCI6MCwidXNlZCI6MX0sImNvc3RfbWljcm9fdXNkIjp7Imtub3dsZWRnZSI6Imtub3duIiwib3ZlcnJ1biI6MCwicmVsZWFzZWQiOjkwMCwidXNlZCI6MTAwfSwiaW52b2NhdGlvbnMiOnsia25vd2xlZGdlIjoia25vd24iLCJvdmVycnVuIjowLCJyZWxlYXNlZCI6MCwidXNlZCI6MX0sInJlc2VydmF0aW9uX2RpZ2VzdCI6IjdlMTdlYzY3MzQ0YmZmZjY5N2NlNmFkMjBmNTczYjhhNDA1MDVkZTllNmZiYTI0OTVjODVmZTFhMmZlN2U2OTUiLCJyZXNlcnZhdGlvbl9pZCI6ImRicl8yYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhIiwic2NoZW1hX3ZlcnNpb24iOiJ2MWFscGhhMSIsInNldHRsZWRfYXRfdW5peF9tcyI6MjMwMCwid2FsbF90aW1lX21zIjp7Imtub3dsZWRnZSI6Imtub3duIiwib3ZlcnJ1biI6MCwicmVsZWFzZWQiOjQwMCwidXNlZCI6MTAwfX0sImNsZWFudXAiOnsia2luZCI6InByb3ZlZF9lbXB0eSIsIm9ic2VydmVkX2F0X3VuaXhfbXMiOjI0MDAsInJlY2VpcHRfZGlnZXN0IjoiM2UzZTNlM2UzZTNlM2UzZTNlM2UzZTNlM2UzZTNlM2UzZTNlM2UzZTNlM2UzZTNlM2UzZTNlM2UzZTNlM2UzZSJ9LCJjcmVkZW50aWFsX3Byb2plY3Rpb25fZGlnZXN0IjoiYWRkMmQwZGRiZDY5MjYyNTMzZjk1MGRjNjM3NGI3ZjI4OTE3ZTM5ZDhiN2FhNGZmNjRjYWY1ZGYzZDVmYjlkZCIsImludGVudF9pZCI6ImRmaV8wZDFjYWY0YzU3MmM3NjBjMGZmNDM2MzUzMDZmNGIyNjBlYTY2OWQyMTk1ZDA1YTA3MTJhN2U3ZTZhYTk5MTA0IiwibGF1bmNoX2dyYW50X2lkIjoiZGZnXzRjM2IwY2NkYjhkMDAwYTk0NTM1NDdlMDc2NjM4NzVjOTA5MDlhMDA0YTk4NGUxZjYyMWU5NjZmMTgwMzY1YjQiLCJwcm9jZXNzIjp7ImVuZGVkX2F0X3VuaXhfbXMiOjIyMDAsIm9ic2VydmF0aW9uX2RpZ2VzdCI6IjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIiLCJzdGFydGVkX2F0X3VuaXhfbXMiOjIwMDAsInN0YXRlIjp7ImNvZGUiOjAsImtpbmQiOiJleGl0ZWQifX0sInByb3Bvc2FsIjp7ImFydGlmYWN0Ijp7ImRpZ2VzdCI6IjExMDUxM2Q3ZDk5MzI1OTU0NDcwMjI4ODBlZDQ1ZThkMWI5MGEwM2Q5Mzk5NTc1OTZmZGJiMGRjZjM3ZDkyMTIiLCJzaXplX2J5dGVzIjo3NDV9LCJraW5kIjoidmFsaWRhdGVkIiwicHJvcG9zYWxfZGlnZXN0IjoiODM4YzZkNTc4ZjhhYzNmZTJhNDI5NTg3Yjc5NDI2NDAwZTQ2OTdlNDVjMzFkMWM2ODJmZjEwYmRmZDE4MzNkNCIsInByb3Bvc2FsX2lkIjoiY250XzMwMzAzMDMwMzAzMDMwMzAzMDMwMzAzMDMwMzAzMDMwMzAzMDMwMzAzMDMwMzAzMDMwMzAzMDMwMzAzMDMwMzAifSwicHJvdmlkZXJfcHJvYmVfb2JzZXJ2YXRpb25fZGlnZXN0IjoiMWQ5ODhlNzJmNjY2ZWVlNjg1OGMxMjhjMTViOWQyZTI0NjA2ZmFmODQyYzAzMDg5ZmI4NTY3OTdmMjhmZTEzZCIsInJlcG9zaXRvcnlfY29udGV4dF9wb3N0X29ic2VydmF0aW9uX2RpZ2VzdCI6Ijc3MjdiYWZiMzc1ZWRhZThjODNhNjVlNjA3YzEyNjQ5OWE4MmI3N2MwNDlmMjQyZTBkY2ZlYTk0MjQ3MGIxNWUiLCJzY2hlbWFfdmVyc2lvbiI6InYxYWxwaGExIiwic3ViamVjdCI6eyJidWRnZXRfcmVzZXJ2YXRpb25faWQiOiJkYnJfMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYTJhMmEyYSIsImRlYWRsaW5lX3VuaXhfbXMiOjMwMDAsImV4ZWN1dGlvbiI6eyJhdHRlbXB0X2ZlbmNlIjozLCJhdHRlbXB0X2lkIjoiYXRtXzE0MTQxNDE0MTQxNDE0MTQxNDE0MTQxNDE0MTQxNDE0MTQxNDE0MTQxNDE0MTQxNDE0MTQxNDE0MTQxNDE0MTQiLCJhdXRob3JpdHlfZXBvY2giOjUsImNvbW1hbmRfaWQiOiJjbWRfMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMCIsImZyZWV6ZV9nZW5lcmF0aW9uIjo2LCJncmFwaF9yZXZpc2lvbl9pZCI6ImdyZl8yMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyMjIyIiwibWlzc2lvbl9pZCI6Im1pc18yMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxIiwicmVwb3NpdG9yeV9pZCI6InJlcF8xMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyMTIxMjEyIiwicnVuX2lkIjoiZGZyXzExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTExMTEiLCJydW5uZXJfZXBvY2giOjQsInJ1bm5lcl9pZCI6InJ1bl8yNTI1MjUyNTI1MjUyNTI1MjUyNTI1MjUyNTI1MjUyNTI1MjUyNTI1MjUyNTI1MjUyNTI1MjUyNTI1MjUyNTI1IiwidmFyaWFudF9pZCI6InZhcl8yNDI0MjQyNDI0MjQyNDI0MjQyNDI0MjQyNDI0MjQyNDI0MjQyNDI0MjQyNDI0MjQyNDI0MjQyNDI0MjQyNDI0Iiwid29ya19wYWNrYWdlX2lkIjoid3BrXzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMyMzIzMjMifSwiZ2F0ZV9pZHMiOlsiZ2F0XzFkMWQxZDFkMWQxZDFkMWQxZDFkMWQxZDFkMWQxZDFkMWQxZDFkMWQxZDFkMWQxZDFkMWQxZDFkMWQxZDFkMWQiLCJnYXRfMWUxZTFlMWUxZTFlMWUxZTFlMWUxZTFlMWUxZTFlMWUxZTFlMWUxZTFlMWUxZTFlMWUxZTFlMWUxZTFlMWUxZSJdLCJvdXRwdXRfc2NoZW1hX2RpZ2VzdCI6IjJiMmIyYjJiMmIyYjJiMmIyYjJiMmIyYjJiMmIyYjJiMmIyYjJiMmIyYjJiMmIyYjJiMmIyYjJiMmIyYjJiMmIiLCJwb2xpY3kiOnsiY29udGFpbm1lbnRfcG9saWN5X2RpZ2VzdCI6IjI5MjkyOTI5MjkyOTI5MjkyOTI5MjkyOTI5MjkyOTI5MjkyOTI5MjkyOTI5MjkyOTI5MjkyOTI5MjkyOTI5MjkiLCJkb2dmb29kX2JpbmRpbmdfZGlnZXN0IjoiMjgyODI4MjgyODI4MjgyODI4MjgyODI4MjgyODI4MjgyODI4MjgyODI4MjgyODI4MjgyODI4MjgyODI4MjgyOCIsImVncmVzc19wb2xpY3lfZGlnZXN0IjoiMDUwNTA1MDUwNTA1MDUwNTA1MDUwNTA1MDUwNTA1MDUwNTA1MDUwNTA1MDUwNTA1MDUwNTA1MDUwNTA1MDUwNSIsInBvbGljeV9nZW5lcmF0aW9uIjoyLCJwb2xpY3lfc25hcHNob3RfZGlnZXN0IjoiZmIyM2I0N2E3M2FhZDRlZGFiN2Q1ZWUwMTY0NGI4YWMyNDcyNWRhODJiMmU0MmY3NWU1MjkxMWRlNGU1MWZhYSIsInRvb2xfcG9saWN5X2RpZ2VzdCI6IjA2MDYwNjA2MDYwNjA2MDYwNjA2MDYwNjA2MDYwNjA2MDYwNjA2MDYwNjA2MDYwNjA2MDYwNjA2MDYwNjA2MDYifSwicHJvbXB0X2RpZ2VzdCI6IjI3MjcyNzI3MjcyNzI3MjcyNzI3MjcyNzI3MjcyNzI3MjcyNzI3MjcyNzI3MjcyNzI3MjcyNzI3MjcyNzI3MjciLCJwcm92aWRlciI6eyJjcmVkZW50aWFsX3Byb2plY3Rpb25faWQiOiJwY3BfMjYyNjI2MjYyNjI2MjYyNjI2MjYyNjI2MjYyNjI2MjYyNjI2MjYyNjI2MjYyNjI2MjYyNjI2MjYyNjI2MjYyNiIsInByb3RvY29sIjoiY2xhdWRlX3N0cmVhbV9qc29uIiwicHJvdmlkZXIiOiJjbGF1ZGUiLCJwcm92aWRlcl9lbnJvbGxtZW50X2lkIjoicGVuXzNkMmQ0ZWUzNjQ3OWIxYjk2MmIxZWQzZjIyMzE0ZmYyOGM3ZGUwY2QwNWFjMDYwYjllMGZkYzA1ZjBkYTc1MDgiLCJwcm92aWRlcl9wcm9maWxlX2lkIjoicHJmXzAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIwMjAyMDIiLCJydW50aW1lX3Bhc3Nwb3J0X2lkIjoicnRwXzMxYmFiZTk5ZjZhZDZmYjIwY2Q0Y2Y1ZjM3NmFjMmY0MTM3ODVkMjBhNmRhZGM4NzdlM2JjNmM0NmYwMmQwN2UifSwicmVwb3NpdG9yeSI6eyJjaGVja3BvaW50X2lkIjoiY2twXzE4MTgxODE4MTgxODE4MTgxODE4MTgxODE4MTgxODE4MTgxODE4MTgxODE4MTgxODE4MTgxODE4MTgxODE4MTgiLCJjb250ZXh0X3NuYXBzaG90X2lkIjoicmNzXzkxYTllODNhMDlhYTZkYWQzZmU5YmE1NzAxYWExZTcwOTAzMzBiOGFmMDdmYWQxYjYyZTM3YTFmYTNhN2IyYzciLCJoZWFkX29pZCI6InNoYTI1NjoxNjE2MTYxNjE2MTYxNjE2MTYxNjE2MTYxNjE2MTYxNjE2MTYxNjE2MTYxNjE2MTYxNjE2MTYxNjE2MTYxNjE2IiwidHJlZV9vaWQiOiJzaGEyNTY6MTcxNzE3MTcxNzE3MTcxNzE3MTcxNzE3MTcxNzE3MTcxNzE3MTcxNzE3MTcxNzE3MTcxNzE3MTcxNzE3MTcxNyJ9fX1fcGvyGH7uWJA4QuiloZuBMFmlZhQmwk_kn3GLpSdNnLHLLj1qswzEf5iawkqPdjjd6R8HWDVBvEvVoUcETmgK.eyJpc3N1ZXIiOiJwcmlfMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMTMxMzEzMSIsImtleV9pZCI6ImRvZ2Zvb2QtcnVuLTEiLCJwdXJwb3NlIjoiZG9nZm9vZC1ydW4tYXR0ZXN0YXRpb24tc2lnbmluZyIsInNjaGVtYV92ZXJzaW9uIjoidjFhbHBoYTEifQ"###;

#[rustfmt::skip]
#[derive(Clone, Deserialize)]
struct Fixture {
    passport: ProviderRuntimePassportV1, enrollment: ProviderEnrollmentClaimsV2,
    intent: DogfoodReadOnlyIntentV1, grant: DogfoodLaunchGrantClaimsV1,
    projection: ProviderCredentialProjectionV1, reservation: DogfoodBudgetReservationV1,
    context: RepositoryContextSnapshotV1, post: RepositoryContextPostObservationV1,
    probe: ProviderProbeObservationV1, endpoint: ProviderEndpointObservationV1,
    version: ProviderVersionObservationV1, profile: ProviderProfileObservationV1,
    proposal: Option<PatchProposal>, run: DogfoodRunV1,
}

fn bytes(raw: &str) -> Vec<u8> {
    hex::decode(raw).unwrap()
}
fn digest(seed: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([seed; 32])
}
#[rustfmt::skip]
fn key_index(policy: &PolicySnapshotV1, purpose: KeyPurposeV1) -> usize {
    policy.issuer_keys.iter().position(|key| key.key_purpose == purpose).unwrap()
}
#[rustfmt::skip]
fn policy() -> PolicySnapshotV1 {
    let mut policy: PolicySnapshotV1 = decode_canonical(POLICY).unwrap();
    policy.activation_at_unix_ms = 1_000;
    policy.expires_at_unix_ms = 1_000_000;
    for key in &mut policy.issuer_keys {
        key.activates_at_unix_ms = 1_000;
        key.expires_at_unix_ms = 1_000_000;
        key.retain_until_unix_ms = 1_015_000;
        key.revoked_at_unix_ms = None;
    }
    let release = key_index(&policy, KeyPurposeV1::ReleaseSigning);
    policy.issuer_keys[release].issuer = RELEASE.into();
    let authority = key_index(&policy, KeyPurposeV1::AuthoritySigning);
    let mut base = policy.issuer_keys[authority].clone();
    policy.issuer_keys[authority].issuer = LIVE.into();
    policy.issuer_keys[authority].key_id = LIVE_KEY.into();
    policy.issuer_keys[authority].public_key = LIVE_PUBLIC.into();
    base.audiences.clear();
    let make = |issuer: &str, key_id: &str, purpose, public: &str| {
        let mut key = base.clone();
        key.issuer = issuer.into(); key.key_id = key_id.into();
        key.key_purpose = purpose; key.public_key = public.into();
        key
    };
    policy.issuer_keys.extend([
        make(LAUNCH, LAUNCH_KEY, KeyPurposeV1::DogfoodLaunchSigning, LAUNCH_PUBLIC),
        make(ENROLL, ENROLL_KEY, KeyPurposeV1::ProviderEnrollmentSigning, ENROLL_PUBLIC),
        make(ATT, RUN_KEY, KeyPurposeV1::DogfoodRunAttestationSigning, RUN_PUBLIC),
    ]);
    policy.validate().unwrap();
    policy
}
fn policy_digest(policy: &PolicySnapshotV1) -> Blake3Digest {
    policy_snapshot_digest(&canonical_json(policy).unwrap()).unwrap()
}
fn passport(provider: LaunchProvider) -> ProviderRuntimePassportV1 {
    let (version, entrypoint) = match provider {
        LaunchProvider::Claude => ("2.1.251", "bin/claude"),
        LaunchProvider::Codex => ("0.150.1", "bin/codex"),
        LaunchProvider::Cursor => ("2026.08.11", "bin/cursor-agent"),
        LaunchProvider::Agy => ("1.1.19", "bin/agy"),
    };
    ProviderRuntimePassportV1 {
        schema_version: 1,
        provider,
        protocol: DogfoodProviderProtocolV1::required_for(provider),
        version: version.into(),
        deployment_root: format!("/usr/lib/bullet/providers/{}/{version}", provider.as_str()),
        entrypoint: entrypoint.into(),
        execution: RuntimeExecutionV1::Native {
            loader: RuntimeLoaderV1::Static,
        },
        files: vec![RuntimeFileV1 {
            path: entrypoint.into(),
            role: RuntimeFileRoleV1::Entrypoint,
            mode: 0o555,
            size: 1,
            blake3: "11".repeat(32),
        }],
        aggregate_file_count: 1,
        aggregate_size_bytes: 1,
    }
}
fn fixture(provider: LaunchProvider, policy: &PolicySnapshotV1) -> Fixture {
    let value = decode_unique_value(BASE_FIXTURE.as_bytes()).unwrap();
    let mut value: Fixture = serde_json::from_value(value).unwrap();
    value.passport = passport(provider);
    let protocol = value.passport.protocol;
    let policy_digest = policy_digest(policy);
    value.enrollment.issuer = ENROLL.into();
    value.enrollment.key_id = ENROLL_KEY.into();
    value.enrollment.provider = provider;
    value.enrollment.protocol = protocol;
    value.enrollment.runtime_passport_id = value.passport.passport_id().unwrap();
    value.enrollment.runtime_version = value.passport.version.clone();
    value.enrollment.policy_snapshot_digest = policy_digest;
    value.enrollment.policy_generation = policy.policy_generation;
    value.probe.subject.provider = provider;
    value.probe.subject.protocol = protocol;
    value.probe.subject.runtime_passport_id = value.enrollment.runtime_passport_id.clone();
    value.probe.subject.policy_snapshot_digest = policy_digest;
    value.probe.subject.policy_generation = policy.policy_generation;
    for subject in [
        &mut value.endpoint.subject,
        &mut value.version.subject,
        &mut value.profile.subject,
    ] {
        *subject = value.probe.subject.clone();
    }
    let probe_digest = value.probe.digest().unwrap();
    value.endpoint.probe_observation_digest = probe_digest;
    value.version.probe_observation_digest = probe_digest;
    value.version.runtime_version = value.passport.version.clone();
    value.profile.probe_observation_digest = probe_digest;
    value.enrollment.endpoint_observation_digest = value.endpoint.digest().unwrap();
    value.enrollment.version_observation_digest = value.version.digest().unwrap();
    value.enrollment.profile_observation_digest = value.profile.digest().unwrap();
    let enrollment_id = value.enrollment.enrollment_id().unwrap();
    value.intent.subject.provider.provider = provider;
    value.intent.subject.provider.protocol = protocol;
    value.intent.subject.provider.runtime_passport_id =
        value.enrollment.runtime_passport_id.clone();
    value.intent.subject.provider.provider_enrollment_id = enrollment_id.clone();
    value.intent.subject.policy.policy_snapshot_digest = policy_digest;
    value.intent.subject.policy.policy_generation = policy.policy_generation;
    value.grant.issuer = LAUNCH.into();
    value.grant.key_id = LAUNCH_KEY.into();
    value.grant.intent_id = value.intent.intent_id().unwrap();
    value.grant.subject = value.intent.subject.clone();
    value.projection.provider = provider;
    value.reservation.provider = provider;
    value.reservation.provider_enrollment_id = enrollment_id;
    value.run.subject = value.intent.subject.clone();
    value.run.intent_id = value.intent.intent_id().unwrap();
    value.run.launch_grant_id = value.grant.grant_id().unwrap();
    value.run.credential_projection_digest = value.projection.projection_digest().unwrap();
    value.run.budget_settlement.reservation_digest =
        value.reservation.reservation_digest().unwrap();
    value.run.provider_probe_observation_digest = probe_digest;
    value
}
#[rustfmt::skip]
impl Fixture {
    fn subjects(&self) -> DogfoodRunBindingSubjects<'_> {
        DogfoodRunBindingSubjects {
            grant: &self.grant, intent: &self.intent, enrollment: &self.enrollment,
            passport: &self.passport, projection: &self.projection, reservation: &self.reservation,
            context_snapshot: &self.context, post_context: &self.post, probe: &self.probe,
            endpoint: &self.endpoint, version: &self.version, profile: &self.profile,
            proposal: self.proposal.as_ref(),
        }
    }
}
fn signer() -> DogfoodRunAttestationSigningKey {
    DogfoodRunAttestationSigningKey::from_bytes(
        &PrincipalId::from_digest(digest(0x31)),
        RUN_KEY,
        &bytes(RUN_SECRET),
    )
    .unwrap()
}
fn signed(value: &Fixture) -> SignedDogfoodRunV1 {
    signer().sign(&value.run, &value.subjects()).unwrap()
}
fn refusal<T>(result: Result<T, WireError>, code: &'static str) -> WireError {
    let error = result.err().unwrap_or_else(|| panic!("expected {code}"));
    assert_eq!(error.code(), code, "{error}");
    error
}
#[rustfmt::skip]
fn raw_signed(payload: &[u8], purpose: &str, assertion: &[u8]) -> SignedDogfoodRunV1 {
    let key = PurposeSeparatedPasetoSigningKey::from_bytes(ATT, RUN_KEY, &bytes(RUN_SECRET)).unwrap();
    let footer = canonical_json(&json!({
        "issuer": ATT,
        "key_id": RUN_KEY,
        "purpose": purpose,
        "schema_version": DOGFOOD_SCHEMA_VERSION,
    }))
    .unwrap();
    SignedDogfoodRunV1 { schema_version: DOGFOOD_SCHEMA_VERSION.into(), issuer: ATT.into(),
        key_id: RUN_KEY.into(), paseto: key.sign(payload, &footer, assertion).unwrap() }
}
#[rustfmt::skip]
fn terminal(value: &mut Fixture, state: DogfoodTerminalStateV1) {
    match state {
        DogfoodTerminalStateV1::ProposalReady => {}
        DogfoodTerminalStateV1::Failed => value.run.process.state = DogfoodProcessStateV1::Exited { code: 1 },
        DogfoodTerminalStateV1::Quarantined => value.run.cleanup = DogfoodCleanupObservationV1::Quarantined {
            receipt_digest: digest(60), residue_manifest_digest: digest(61), observed_at_unix_ms: 2_400,
        },
        DogfoodTerminalStateV1::OutcomeUnknown => {
            value.run.process.state = DogfoodProcessStateV1::OutcomeUnknown;
            value.proposal = None;
            value.run.proposal = DogfoodProposalObservationV1::Absent;
            value.run.budget_settlement.cost_micro_usd = DogfoodUsageSettlementV1::Unknown { retained: value.reservation.reserved_cost_micro_usd };
            value.run.budget_settlement.invocations = DogfoodUsageSettlementV1::Unknown { retained: value.reservation.reserved_invocations };
            value.run.budget_settlement.wall_time_ms = DogfoodUsageSettlementV1::Unknown { retained: value.reservation.reserved_wall_time_ms };
            value.run.budget_settlement.concurrency = DogfoodUsageSettlementV1::Unknown { retained: value.reservation.reserved_concurrency };
        }
        DogfoodTerminalStateV1::RefusedBeforeSpawn => {
            value.run.process.state = DogfoodProcessStateV1::NotStarted;
            value.run.process.started_at_unix_ms = None;
            value.run.process.ended_at_unix_ms = None;
            value.proposal = None;
            value.run.proposal = DogfoodProposalObservationV1::Absent;
            value.run.budget_settlement.cost_micro_usd = DogfoodUsageSettlementV1::Known { used: 0, released: value.reservation.reserved_cost_micro_usd, overrun: 0 };
            value.run.budget_settlement.invocations = DogfoodUsageSettlementV1::Known { used: 0, released: value.reservation.reserved_invocations, overrun: 0 };
            value.run.budget_settlement.wall_time_ms = DogfoodUsageSettlementV1::Known { used: 0, released: value.reservation.reserved_wall_time_ms, overrun: 0 };
            value.run.budget_settlement.concurrency = DogfoodUsageSettlementV1::Known { used: 0, released: value.reservation.reserved_concurrency, overrun: 0 };
        }
    }
}

#[rustfmt::skip]
#[test]
fn all_provider_terminal_pairs_round_trip_under_literal_domains() {
    assert_eq!(
        DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE,
        "dogfood-run-attestation-signing"
    );
    assert_eq!(
        DOGFOOD_RUN_ATTESTATION_IMPLICIT_ASSERTION,
        b"bullet-farm.dogfood-run-attestation.v1alpha1"
    );
    assert_eq!(
        DOGFOOD_RUN_ATTESTATION_ENVELOPE_DOMAIN,
        "dogfood.run-attestation-envelope.v1alpha1"
    );
    assert_eq!(
        (
            MAX_DOGFOOD_RUN_ATTESTATION_TOKEN_BYTES,
            MAX_DOGFOOD_RUN_ATTESTATION_AGE_MS
        ),
        (96 * 1024, 300_000)
    );
    assert_eq!(
        canonical_json(&footer(ATT, RUN_KEY)).unwrap(),
        br#"{"issuer":"pri_3131313131313131313131313131313131313131313131313131313131313131","key_id":"dogfood-run-1","purpose":"dogfood-run-attestation-signing","schema_version":"v1alpha1"}"#
    );
    let policy = policy();
    let mut immutable_body_and_compact_token_commitments = Vec::new();
    for provider in [
        LaunchProvider::Claude,
        LaunchProvider::Codex,
        LaunchProvider::Cursor,
        LaunchProvider::Agy,
    ] {
        assert_eq!(fixture(provider, &policy).enrollment.protocol, DogfoodProviderProtocolV1::required_for(provider));
        for state in [
            DogfoodTerminalStateV1::ProposalReady,
            DogfoodTerminalStateV1::Failed,
            DogfoodTerminalStateV1::Quarantined,
            DogfoodTerminalStateV1::OutcomeUnknown,
            DogfoodTerminalStateV1::RefusedBeforeSpawn,
        ] {
            let mut value = fixture(provider, &policy);
            terminal(&mut value, state);
            assert_eq!(value.run.terminal_state(&value.reservation).unwrap(), state);
            let envelope = signed(&value);
            if provider == LaunchProvider::Claude && state == DogfoodTerminalStateV1::ProposalReady {
                assert_eq!(canonical_json(&value.run).unwrap(), GOLDEN_CLAUDE_PROPOSAL_READY_RUN);
                assert_eq!(envelope.paseto, GOLDEN_CLAUDE_PROPOSAL_READY_TOKEN);
            }
            assert_eq!(envelope.digest().unwrap(), hash_framed_bytes(DOGFOOD_RUN_ATTESTATION_ENVELOPE_DOMAIN, envelope.paseto.as_bytes()).unwrap());
            assert_eq!(
                envelope.verify(&policy, &value.subjects(), NOW).unwrap(),
                value.run
            );
            assert_eq!(
                envelope,
                raw_signed(
                    &canonical_json(&value.run).unwrap(),
                    DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE,
                    DOGFOOD_RUN_ATTESTATION_IMPLICIT_ASSERTION
                )
            );
            if state == DogfoodTerminalStateV1::ProposalReady {
                immutable_body_and_compact_token_commitments.push((
                    value.run.digest().unwrap().to_string(),
                    envelope.digest().unwrap().to_string(),
                ));
            }
        }
    }
    assert_eq!(immutable_body_and_compact_token_commitments, [
        ("69e0e49685a65f9324e6f166218025a07fcf841f1746399cbfd4962576e529ff", "224fb787a1f5627396221739435de073f1ab4ca1cd0fbc99ff966d4c4c34a038"),
        ("a175b019378e542a45fb19b628aa21b23255af00c0dd8d3414b0b4b2580288f8", "1036f78157e7d328c8688c111c65766e3fbaa9e4f85e96d44a0d1c1ad07e9532"),
        ("c56a5e5772f8f67563572dc2c03bd9db19503e1a9e900e01d7d2dfce92e69e5c", "2d7cb2282c6f10a02d575977b19a4cd2400003972994fb54df3ebad679c7bc22"),
        ("b59da5c12bcf7e4807540acae6737d19bae2fffcc105f080ec45597b12cd6f9c", "fd91e57ce102d9f924b2eb4fcb17017adb39ee9d4dcb7c7dc798e75f14ab33b3"),
    ].map(|(run, envelope)| (run.to_owned(), envelope.to_owned())));
}

#[rustfmt::skip]
#[test]
fn policy_selection_identity_loaded_body_and_error_order_are_exact() {
    let policy = policy();
    let value = fixture(LaunchProvider::Codex, &policy);
    let envelope = signed(&value);
    let mut unknown = envelope.clone();
    unknown.key_id = "unknown".into();
    refusal(unknown.verify(&policy, &value.subjects(), NOW), "DOGFOOD_RUN_ATTESTOR_KEY_UNKNOWN");
    for purpose in [KeyPurposeV1::DogfoodLaunchSigning, KeyPurposeV1::ProviderEnrollmentSigning, KeyPurposeV1::AuthoritySigning, KeyPurposeV1::ReleaseSigning] {
        let key = &policy.issuer_keys[key_index(&policy, purpose)];
        let mut crossed = envelope.clone();
        crossed.issuer = key.issuer.clone();
        crossed.key_id = key.key_id.clone();
        refusal(crossed.verify(&policy, &value.subjects(), NOW), "DOGFOOD_RUN_ATTESTOR_KEY_WRONG_PURPOSE");
    }
    let mut same_generation = policy.clone();
    same_generation.budget_policy.maximum_changed_paths -= 1;
    same_generation.validate().unwrap();
    refusal(envelope.verify(&same_generation, &value.subjects(), NOW), "DOGFOOD_RUN_POLICY_MISMATCH");
    let mut generation = policy.clone();
    generation.policy_generation += 1;
    refusal(envelope.verify(&generation, &value.subjects(), NOW), "DOGFOOD_RUN_POLICY_MISMATCH");
    for (mutate, code) in [
        ((|p: &mut PolicySnapshotV1| p.evidence_policy.unknown_satisfies_gate = true) as fn(&mut PolicySnapshotV1), "UNSAFE_POLICY"),
        (|p| { let i = key_index(p, KeyPurposeV1::DogfoodRunAttestationSigning); p.issuer_keys[i].public_key = "AA".repeat(32); }, "INVALID_DOGFOOD_RUN_ATTESTOR_PUBLIC_KEY"),
        (|p| { let i = key_index(p, KeyPurposeV1::DogfoodRunAttestationSigning); p.issuer_keys[i].public_key = LIVE_PUBLIC.into(); }, "SIGNER_KEY_MATERIAL_REUSED"),
        (|p| { let i = key_index(p, KeyPurposeV1::DogfoodRunAttestationSigning); p.issuer_keys[i].expires_at_unix_ms = p.issuer_keys[i].activates_at_unix_ms; }, "INVALID_ISSUER_KEY_LIFECYCLE"),
        (|p| { let i = key_index(p, KeyPurposeV1::DogfoodRunAttestationSigning); p.issuer_keys[i].algorithm = KeyAlgorithmV1::SshEd25519; }, "INVALID_DOGFOOD_RUN_ATTESTOR_PUBLIC_KEY"),
        (|p| { let i = key_index(p, KeyPurposeV1::DogfoodRunAttestationSigning); p.issuer_keys[i].audiences = vec![AuthorityAudience::ProviderRunner]; }, "INVALID_DOGFOOD_RUN_ATTESTOR_PUBLIC_KEY"),
    ] {
        let mut hostile = policy.clone();
        mutate(&mut hostile);
        refusal(envelope.verify(&hostile, &value.subjects(), NOW), code);
    }
    let mut other = value.run.clone();
    other.attestor_principal_id = PrincipalId::from_digest(digest(50));
    refusal(raw_signed(&canonical_json(&other).unwrap(), DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE, DOGFOOD_RUN_ATTESTATION_IMPLICIT_ASSERTION).verify(&policy, &value.subjects(), NOW), "DOGFOOD_RUN_ATTESTOR_MISMATCH");
    let mut invalid_auth = envelope.clone();
    invalid_auth.paseto.push('A');
    refusal(invalid_auth.verify(&same_generation, &value.subjects(), NOW), "DOGFOOD_RUN_ATTESTATION_INVALID");
    refusal(invalid_auth.verify(&policy, &value.subjects(), policy.activation_at_unix_ms - 1), "POLICY_NOT_ACTIVE");
    let other_signer = DogfoodRunAttestationSigningKey::from_bytes(&PrincipalId::from_digest(digest(50)), RUN_KEY, &bytes(RUN_SECRET)).unwrap();
    refusal(other_signer.sign(&value.run, &value.subjects()), "DOGFOOD_RUN_ATTESTOR_MISMATCH");
    let mut bad_binding = value.clone();
    bad_binding.intent.request_digest = digest(200);
    refusal(envelope.verify(&policy, &bad_binding.subjects(), NOW), "DOGFOOD_GRANT_SUBJECT_MISMATCH");
}

#[rustfmt::skip]
#[test]
fn trusted_freshness_and_historical_lifecycle_boundaries_are_exact() {
    let policy = policy();
    let value = fixture(LaunchProvider::Claude, &policy);
    let envelope = signed(&value);
    refusal(envelope.verify(&policy, &value.subjects(), NOW - 1), "DOGFOOD_RUN_ATTESTATION_IN_FUTURE");
    envelope.verify(&policy, &value.subjects(), NOW).unwrap();
    envelope.verify(&policy, &value.subjects(), NOW + MAX_DOGFOOD_RUN_ATTESTATION_AGE_MS).unwrap();
    refusal(envelope.verify(&policy, &value.subjects(), NOW + MAX_DOGFOOD_RUN_ATTESTATION_AGE_MS + 1), "DOGFOOD_RUN_ATTESTATION_STALE");
    refusal(envelope.verify(&policy, &value.subjects(), policy.activation_at_unix_ms - 1), "POLICY_NOT_ACTIVE");
    refusal(envelope.verify(&policy, &value.subjects(), policy.expires_at_unix_ms), "POLICY_NOT_ACTIVE");
    let index = key_index(&policy, KeyPurposeV1::DogfoodRunAttestationSigning);
    for (activation, now, code) in [(NOW + 1, NOW, "DOGFOOD_RUN_ATTESTOR_KEY_INACTIVE"), (NOW, NOW, "")] {
        let mut boundary = policy.clone();
        boundary.issuer_keys[index].activates_at_unix_ms = activation;
        let rebound = fixture(LaunchProvider::Claude, &boundary);
        let token = signed(&rebound);
        if code.is_empty() { token.verify(&boundary, &rebound.subjects(), now).unwrap(); }
        else { refusal(token.verify(&boundary, &rebound.subjects(), now), code); }
    }
    let mut historical = policy.clone();
    historical.issuer_keys[index].activates_at_unix_ms = NOW + 1;
    let rebound = fixture(LaunchProvider::Claude, &historical);
    refusal(signed(&rebound).verify(&historical, &rebound.subjects(), NOW + 1), "DOGFOOD_RUN_ATTESTOR_KEY_INACTIVE");
    let mut historical_policy = policy.clone();
    historical_policy.activation_at_unix_ms = NOW + 1;
    let rebound = fixture(LaunchProvider::Claude, &historical_policy);
    refusal(signed(&rebound).verify(&historical_policy, &rebound.subjects(), NOW + 1), "POLICY_NOT_ACTIVE");
    let mut revoked = policy.clone();
    revoked.issuer_keys[index].revoked_at_unix_ms = Some(NOW);
    refusal(envelope.verify(&revoked, &value.subjects(), NOW), "DOGFOOD_RUN_ATTESTOR_KEY_INACTIVE");
    let mut expired = policy.clone();
    expired.issuer_keys[index].expires_at_unix_ms = NOW;
    refusal(envelope.verify(&expired, &value.subjects(), NOW), "DOGFOOD_RUN_ATTESTOR_KEY_INACTIVE");
    for (mutate, code) in [
        ((|p: &mut PolicySnapshotV1| p.expires_at_unix_ms = 10_000) as fn(&mut PolicySnapshotV1), "POLICY_NOT_ACTIVE"),
        (|p| { let i = key_index(p, KeyPurposeV1::DogfoodRunAttestationSigning); p.issuer_keys[i].expires_at_unix_ms = 10_000; }, "DOGFOOD_RUN_ATTESTOR_KEY_INACTIVE"),
        (|p| { let i = key_index(p, KeyPurposeV1::DogfoodRunAttestationSigning); p.issuer_keys[i].revoked_at_unix_ms = Some(10_000); }, "DOGFOOD_RUN_ATTESTOR_KEY_INACTIVE"),
    ] {
        let mut boundary = policy.clone(); mutate(&mut boundary); boundary.validate().unwrap();
        let mut late = fixture(LaunchProvider::Claude, &boundary); terminal(&mut late, DogfoodTerminalStateV1::Failed); late.run.attested_at_unix_ms = 9_999;
        let token = signed(&late); token.verify(&boundary, &late.subjects(), 9_999).unwrap();
        refusal(token.verify(&boundary, &late.subjects(), 10_000), code);
        let mut at_boundary = fixture(LaunchProvider::Claude, &boundary); terminal(&mut at_boundary, DogfoodTerminalStateV1::Failed); at_boundary.run.attested_at_unix_ms = 10_000;
        refusal(signed(&at_boundary).verify(&boundary, &at_boundary.subjects(), 10_000), code);
    }
    let mut activates = policy; activates.activation_at_unix_ms = NOW;
    let exact = fixture(LaunchProvider::Claude, &activates); let token = signed(&exact);
    refusal(token.verify(&activates, &exact.subjects(), NOW - 1), "POLICY_NOT_ACTIVE");
    token.verify(&activates, &exact.subjects(), NOW).unwrap();
}

#[rustfmt::skip]
#[test]
fn every_w0_through_w5_subject_is_rechecked_after_authentication() {
    let policy = policy();
    let base = fixture(LaunchProvider::Cursor, &policy);
    let envelope = signed(&base);
    let cases: &[MutationCase<Fixture>] = &[
        (|v| v.grant.request_digest = digest(60), "DOGFOOD_GRANT_SUBJECT_MISMATCH"),
        (|v| v.intent.request_digest = digest(61), "DOGFOOD_GRANT_SUBJECT_MISMATCH"),
        (|v| v.enrollment.service_identity_id = PrincipalId::from_digest(digest(62)), "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH"),
        (|v| v.enrollment.provider = LaunchProvider::Claude, "DOGFOOD_PROVIDER_PROTOCOL_MISMATCH"),
        (|v| v.enrollment.provider_profile_id = ProviderProfileId::from_digest(digest(63)), "PROVIDER_ENROLLMENT_SUBJECT_MISMATCH"),
        (|v| { v.passport.version = "changed".into(); v.passport.deployment_root = "/usr/lib/bullet/providers/cursor/changed".into(); }, "RUNTIME_PASSPORT_ID_MISMATCH"),
        (|v| v.projection.target_policy_digest = digest(64), "DOGFOOD_RUN_RESOURCE_MISMATCH"),
        (|v| v.projection.projection_instance_id = ProviderCredentialProjectionId::from_digest(digest(71)), "DOGFOOD_RUN_RESOURCE_MISMATCH"),
        (|v| v.projection.credential_projection_profile_id = CredentialProjectionProfileId::from_digest(digest(72)), "DOGFOOD_RUN_RESOURCE_MISMATCH"),
        (|v| v.reservation.budget_policy_digest = digest(65), "DOGFOOD_BUDGET_SUBJECT_MISMATCH"),
        (|v| v.context.scope_grant_digest = digest(66), "REPOSITORY_CONTEXT_ID_MISMATCH"),
        (|v| v.context.attempt_id = AttemptId::from_digest(digest(73)), "REPOSITORY_CONTEXT_ID_MISMATCH"),
        (|v| v.context.attempt_fence += 1, "REPOSITORY_CONTEXT_ID_MISMATCH"),
        (|v| v.context.head_oid = GitOid::Sha256("74".repeat(32)), "REPOSITORY_CONTEXT_ID_MISMATCH"),
        (|v| v.context.tree_oid = GitOid::Sha256("75".repeat(32)), "REPOSITORY_CONTEXT_ID_MISMATCH"),
        (|v| v.context.checkpoint_id = CheckpointId::from_digest(digest(76)), "REPOSITORY_CONTEXT_ID_MISMATCH"),
        (|v| v.post.observed_checkpoint_digest = digest(67), "REPOSITORY_CONTEXT_POST_MISMATCH"),
        (|v| v.probe.probe_grant_digest = digest(68), "PROVIDER_PROBE_OBSERVATION_MISMATCH"),
        (|v| v.endpoint.entrypoint_blake3 = digest(69), "PROVIDER_ENDPOINT_OBSERVATION_MISMATCH"),
        (|v| v.version.runtime_version = "changed".into(), "PROVIDER_VERSION_OBSERVATION_MISMATCH"),
        (|v| v.profile.effective_identity_artifact_digest = digest(70), "PROVIDER_PROFILE_OBSERVATION_MISMATCH"),
        (|v| { let PatchMutation::Write { content_utf8 } = &mut v.proposal.as_mut().unwrap().operations[0].mutation else { unreachable!() }; content_utf8.push('x'); }, "DOGFOOD_RUN_PROPOSAL_MISMATCH"),
    ];
    for (mutate, code) in cases {
        let mut value = base.clone();
        mutate(&mut value);
        refusal(envelope.verify(&policy, &value.subjects(), NOW), code);
    }
    let run_cases: &[MutationCase<DogfoodRunV1>] = &[
        (|v| v.subject.execution.attempt_id = AttemptId::from_digest(digest(80)), "DOGFOOD_RUN_SUBJECT_MISMATCH"),
        (|v| v.subject.execution.attempt_fence += 1, "DOGFOOD_RUN_SUBJECT_MISMATCH"),
        (|v| v.subject.provider.provider = LaunchProvider::Claude, "DOGFOOD_PROVIDER_PROTOCOL_MISMATCH"),
        (|v| v.subject.provider.protocol = DogfoodProviderProtocolV1::ClaudeStreamJson, "DOGFOOD_PROVIDER_PROTOCOL_MISMATCH"),
        (|v| v.subject.provider.provider_profile_id = ProviderProfileId::from_digest(digest(81)), "DOGFOOD_RUN_SUBJECT_MISMATCH"),
        (|v| v.subject.provider.provider_enrollment_id = ProviderEnrollmentId::from_digest(digest(82)), "DOGFOOD_RUN_SUBJECT_MISMATCH"),
        (|v| v.subject.provider.credential_projection_id = ProviderCredentialProjectionId::from_digest(digest(83)), "DOGFOOD_RUN_SUBJECT_MISMATCH"),
        (|v| v.subject.repository.head_oid = GitOid::Sha256("84".repeat(32)), "DOGFOOD_RUN_SUBJECT_MISMATCH"),
        (|v| v.subject.repository.tree_oid = GitOid::Sha256("85".repeat(32)), "DOGFOOD_RUN_SUBJECT_MISMATCH"),
        (|v| v.subject.repository.checkpoint_id = CheckpointId::from_digest(digest(86)), "DOGFOOD_RUN_SUBJECT_MISMATCH"),
        (|v| v.subject.policy.dogfood_binding_digest = digest(87), "DOGFOOD_RUN_SUBJECT_MISMATCH"),
        (|v| v.budget_settlement.reservation_id = DogfoodBudgetReservationId::from_digest(digest(88)), "DOGFOOD_RUN_SETTLEMENT_MISMATCH"),
        (|v| { let DogfoodProposalObservationV1::Validated { artifact, .. } = &mut v.proposal else { unreachable!() }; artifact.digest = digest(89); }, "DOGFOOD_RUN_PROPOSAL_MISMATCH"),
        (|v| { let DogfoodProposalObservationV1::Validated { artifact, .. } = &mut v.proposal else { unreachable!() }; artifact.size_bytes += 1; }, "DOGFOOD_RUN_PROPOSAL_MISMATCH"),
        (|v| v.process.ended_at_unix_ms = Some(3_001), "DOGFOOD_RUN_TIME_MISMATCH"),
        (|v| v.cleanup = DogfoodCleanupObservationV1::ProvedEmpty { receipt_digest: digest(90), observed_at_unix_ms: 2_199 }, "DOGFOOD_RUN_TIME_MISMATCH"),
    ];
    for (mutate, code) in run_cases { let mut run = base.run.clone(); mutate(&mut run);
        refusal(raw_signed(&canonical_json(&run).unwrap(), DOGFOOD_RUN_ATTESTATION_SIGNING_PURPOSE, DOGFOOD_RUN_ATTESTATION_IMPLICIT_ASSERTION).verify(&policy, &base.subjects(), NOW), code); }
}
