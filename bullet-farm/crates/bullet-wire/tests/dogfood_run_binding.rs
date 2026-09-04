use bullet_wire::*;
use serde::Deserialize;

const PROVIDERS: [LaunchProvider; 4] = [
    LaunchProvider::Claude,
    LaunchProvider::Codex,
    LaunchProvider::Cursor,
    LaunchProvider::Agy,
];
const BASE_FIXTURE: &str = concat!(
    r#"{"passport":{"schema_version":1,"provider":"claude","protocol":"claude_stream_json","version":"2.1.251","deployment_root":"/usr/lib/bullet/providers/claude/2.1.251","entrypoint":"bin/claude","execution":{"kind":"native","loader":{"kind":"static"}},"files":[{"path":"bin/claude","role":"entrypoint","mode":365,"size":1,"blake3":"1111111111111111111111111111111111111111111111111111111111111111"}],"aggregate_file_count":1,"aggregate_size_bytes":1},"enrollment":{"schema_version":"v1alpha1","issuer":"operator.example","key_id":"provider-enrollment-alpha","signing_purpose":"provider-enrollment-signing","claims_domain":"provider.enrollment-claims.v2","provider":"claude","protocol":"claude_stream_json","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","service_identity_id":"pri_0303030303030303030303030303030303030303030303030303030303030303","credential_projection_profile_id":"cpp_0404040404040404040404040404040404040404040404040404040404040404","runtime_version":"2.1.251","enrollment_generation":2,"activates_at_unix_ms":1000,"expires_at_unix_ms":5000,"revoked_at_unix_ms":null,"egress_policy_digest":"0505050505050505050505050505050505050505050505050505050505050505","tool_policy_digest":"0606060606060606060606060606060606060606060606060606060606060606","budget_policy_digest":"0707070707070707070707070707070707070707070707070707070707070707","endpoint_observation_digest":"bdc3fbc09c5d29de0a65ecfac5268ed6c78b6d1f55828d1027feb88986eae9b3","version_observation_digest":"ea68136c1ee268deec26242eb290f754804239638756e40a9e68700c212f54f3","profile_observation_digest":"0116aa96c7d129e9a351d09c0fb317852fe69bbb230fc795833d925f28777900","policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2},"#,
    r#""intent":{"schema_version":"v1alpha1","request_digest":"1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f","subject":{"execution":{"command_id":"cmd_2020202020202020202020202020202020202020202020202020202020202020","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","mission_id":"mis_2121212121212121212121212121212121212121212121212121212121212121","repository_id":"rep_1212121212121212121212121212121212121212121212121212121212121212","graph_revision_id":"grf_2222222222222222222222222222222222222222222222222222222222222222","work_package_id":"wpk_2323232323232323232323232323232323232323232323232323232323232323","variant_id":"var_2424242424242424242424242424242424242424242424242424242424242424","attempt_id":"atm_1414141414141414141414141414141414141414141414141414141414141414","attempt_fence":3,"runner_id":"run_2525252525252525252525252525252525252525252525252525252525252525","runner_epoch":4,"authority_epoch":5,"freeze_generation":6},"provider":{"provider":"claude","protocol":"claude_stream_json","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_enrollment_id":"pen_951b5254b7d1c170f15dbbd9dd09ca484677d896846a1730fd44b49dc19beae7","credential_projection_id":"pcp_2626262626262626262626262626262626262626262626262626262626262626"},"repository":{"context_snapshot_id":"rcs_91a9e83a09aa6dad3fe9ba5701aa1e7090330b8af07fad1b62e37a1fa3a7b2c7","head_oid":"sha256:1616161616161616161616161616161616161616161616161616161616161616","tree_oid":"sha256:1717171717171717171717171717171717171717171717171717171717171717","checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818"},"gate_ids":["gat_1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d","gat_1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e"],"prompt_digest":"2727272727272727272727272727272727272727272727272727272727272727","policy":{"policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2,"dogfood_binding_digest":"2828282828282828282828282828282828282828282828282828282828282828","tool_policy_digest":"0606060606060606060606060606060606060606060606060606060606060606","egress_policy_digest":"0505050505050505050505050505050505050505050505050505050505050505","containment_policy_digest":"2929292929292929292929292929292929292929292929292929292929292929"},"budget_reservation_id":"dbr_2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a","deadline_unix_ms":3000,"output_schema_digest":"2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"}},"grant":{"schema_version":"v1alpha1","audience":"dogfood-runner","operation":"read-only-propose","issuer":"kernel.example","key_id":"dogfood-launch-alpha","signing_purpose":"dogfood-launch-signing","claims_domain":"authority.dogfood-launch-grant-claims.v1alpha1","issued_at_unix_ms":1900,"not_before_unix_ms":2000,"expires_at_unix_ms":2800,"grant_nonce":"2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c2c","request_digest":"1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f1f","intent_id":"dfi_c84d3bc08ac144e2b99c7537c332fa4d286c0e10719e374f1c428fe305fd5b23","subject":{"execution":{"command_id":"cmd_2020202020202020202020202020202020202020202020202020202020202020","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","mission_id":"mis_2121212121212121212121212121212121212121212121212121212121212121","repository_id":"rep_1212121212121212121212121212121212121212121212121212121212121212","graph_revision_id":"grf_2222222222222222222222222222222222222222222222222222222222222222","work_package_id":"wpk_2323232323232323232323232323232323232323232323232323232323232323","variant_id":"var_2424242424242424242424242424242424242424242424242424242424242424","attempt_id":"atm_1414141414141414141414141414141414141414141414141414141414141414","attempt_fence":3,"runner_id":"run_2525252525252525252525252525252525252525252525252525252525252525","runner_epoch":4,"authority_epoch":5,"freeze_generation":6},"provider":{"provider":"claude","protocol":"claude_stream_json","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_enrollment_id":"pen_951b5254b7d1c170f15dbbd9dd09ca484677d896846a1730fd44b49dc19beae7","credential_projection_id":"pcp_2626262626262626262626262626262626262626262626262626262626262626"},"repository":{"context_snapshot_id":"rcs_91a9e83a09aa6dad3fe9ba5701aa1e7090330b8af07fad1b62e37a1fa3a7b2c7","head_oid":"sha256:1616161616161616161616161616161616161616161616161616161616161616","tree_oid":"sha256:1717171717171717171717171717171717171717171717171717171717171717","checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818"},"gate_ids":["gat_1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d","gat_1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e"],"prompt_digest":"2727272727272727272727272727272727272727272727272727272727272727","policy":{"policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2,"dogfood_binding_digest":"2828282828282828282828282828282828282828282828282828282828282828","tool_policy_digest":"0606060606060606060606060606060606060606060606060606060606060606","egress_policy_digest":"0505050505050505050505050505050505050505050505050505050505050505","containment_policy_digest":"2929292929292929292929292929292929292929292929292929292929292929"},"budget_reservation_id":"dbr_2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a","deadline_unix_ms":3000,"output_schema_digest":"2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"}},"#,
    r#""projection":{"schema_version":"v1alpha1","projection_instance_id":"pcp_2626262626262626262626262626262626262626262626262626262626262626","credential_projection_profile_id":"cpp_0404040404040404040404040404040404040404040404040404040404040404","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","provider":"claude","service_identity_id":"pri_0303030303030303030303030303030303030303030303030303030303030303","activates_at_unix_ms":2000,"expires_at_unix_ms":2800,"target_policy_digest":"2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d2d","secret_commitment_digest":"2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e2e"},"reservation":{"schema_version":"v1alpha1","reservation_id":"dbr_2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","provider":"claude","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","provider_enrollment_id":"pen_951b5254b7d1c170f15dbbd9dd09ca484677d896846a1730fd44b49dc19beae7","budget_policy_digest":"0707070707070707070707070707070707070707070707070707070707070707","reserved_at_unix_ms":2000,"consume_before_unix_ms":2800,"reserved_cost_micro_usd":1000,"reserved_invocations":1,"reserved_wall_time_ms":500,"reserved_concurrency":1},"#,
    r#""context":{"schema_version":"v1alpha1","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","repository_id":"rep_1212121212121212121212121212121212121212121212121212121212121212","source_descriptor_id":"src_1313131313131313131313131313131313131313131313131313131313131313","attempt_id":"atm_1414141414141414141414141414141414141414141414141414141414141414","attempt_fence":3,"owner_principal_id":"pri_1515151515151515151515151515151515151515151515151515151515151515","head_oid":"sha256:1616161616161616161616161616161616161616161616161616161616161616","tree_oid":"sha256:1717171717171717171717171717171717171717171717171717171717171717","checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818","checkpoint_digest":"1919191919191919191919191919191919191919191919191919191919191919","scope_grant_digest":"1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a1a","visible_scopes":["src"],"files":[{"path":"src/lib.rs","preimage_digest":"1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b1b","size_bytes":10,"executable":false}],"aggregate_file_count":1,"aggregate_size_bytes":10,"visible_manifest_digest":"ddf77c46f0528a03057b96cb51ee08c492a87caf169a00c6463fe76e656ba232","prepared_at_unix_ms":2000},"post":{"schema_version":"v1alpha1","context_snapshot_id":"rcs_91a9e83a09aa6dad3fe9ba5701aa1e7090330b8af07fad1b62e37a1fa3a7b2c7","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","observer_principal_id":"pri_2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f2f","observed_at_unix_ms":2200,"observed_owner_principal_id":"pri_1515151515151515151515151515151515151515151515151515151515151515","observed_head_oid":"sha256:1616161616161616161616161616161616161616161616161616161616161616","observed_tree_oid":"sha256:1717171717171717171717171717171717171717171717171717171717171717","observed_checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818","observed_checkpoint_digest":"1919191919191919191919191919191919191919191919191919191919191919","observed_visible_manifest_digest":"ddf77c46f0528a03057b96cb51ee08c492a87caf169a00c6463fe76e656ba232"},"#,
    r#""probe":{"schema_version":"v1alpha1","subject":{"provider":"claude","protocol":"claude_stream_json","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","service_identity_id":"pri_0303030303030303030303030303030303030303030303030303030303030303","credential_projection_profile_id":"cpp_0404040404040404040404040404040404040404040404040404040404040404","policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2},"probe_grant_digest":"0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c0c","containment_receipt_digest":"0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d0d","protocol_transcript_digest":"0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e0e","observed_at_unix_ms":900},"endpoint":{"schema_version":"v1alpha1","subject":{"provider":"claude","protocol":"claude_stream_json","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","service_identity_id":"pri_0303030303030303030303030303030303030303030303030303030303030303","credential_projection_profile_id":"cpp_0404040404040404040404040404040404040404040404040404040404040404","policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2},"probe_observation_digest":"9ee793b9cb4124b834d783d3053986f5c90da8aaf093eb46c899aa398f5248ea","entrypoint_blake3":"1111111111111111111111111111111111111111111111111111111111111111"},"#,
    r#""version":{"schema_version":"v1alpha1","subject":{"provider":"claude","protocol":"claude_stream_json","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","service_identity_id":"pri_0303030303030303030303030303030303030303030303030303030303030303","credential_projection_profile_id":"cpp_0404040404040404040404040404040404040404040404040404040404040404","policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2},"probe_observation_digest":"9ee793b9cb4124b834d783d3053986f5c90da8aaf093eb46c899aa398f5248ea","runtime_version":"2.1.251","native_version_artifact_digest":"0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f0f"},"profile":{"schema_version":"v1alpha1","subject":{"provider":"claude","protocol":"claude_stream_json","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","service_identity_id":"pri_0303030303030303030303030303030303030303030303030303030303030303","credential_projection_profile_id":"cpp_0404040404040404040404040404040404040404040404040404040404040404","policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2},"probe_observation_digest":"9ee793b9cb4124b834d783d3053986f5c90da8aaf093eb46c899aa398f5248ea","effective_identity_artifact_digest":"1010101010101010101010101010101010101010101010101010101010101010"},"#,
    r#""proposal":{"schema_version":1,"proposal_id":"cnt_3030303030303030303030303030303030303030303030303030303030303030","producing_attempt_id":"atm_1414141414141414141414141414141414141414141414141414141414141414","base_checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818","base_checkpoint_digest":"1919191919191919191919191919191919191919191919191919191919191919","operations":[{"path":"src/file-000.rs","preimage":{"kind":"digest","digest":"3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f"},"mutation":{"kind":"write","content_utf8":"fn main() {}\n"}}],"gate_ids":["gat_1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d","gat_1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e"]},"run":{"schema_version":"v1alpha1","subject":{"execution":{"command_id":"cmd_2020202020202020202020202020202020202020202020202020202020202020","run_id":"dfr_1111111111111111111111111111111111111111111111111111111111111111","mission_id":"mis_2121212121212121212121212121212121212121212121212121212121212121","repository_id":"rep_1212121212121212121212121212121212121212121212121212121212121212","graph_revision_id":"grf_2222222222222222222222222222222222222222222222222222222222222222","work_package_id":"wpk_2323232323232323232323232323232323232323232323232323232323232323","variant_id":"var_2424242424242424242424242424242424242424242424242424242424242424","attempt_id":"atm_1414141414141414141414141414141414141414141414141414141414141414","attempt_fence":3,"runner_id":"run_2525252525252525252525252525252525252525252525252525252525252525","runner_epoch":4,"authority_epoch":5,"freeze_generation":6},"provider":{"provider":"claude","protocol":"claude_stream_json","provider_profile_id":"prf_0202020202020202020202020202020202020202020202020202020202020202","runtime_passport_id":"rtp_31babe99f6ad6fb20cd4cf5f376ac2f413785d20a6dadc877e3bc6c46f02d07e","provider_enrollment_id":"pen_951b5254b7d1c170f15dbbd9dd09ca484677d896846a1730fd44b49dc19beae7","credential_projection_id":"pcp_2626262626262626262626262626262626262626262626262626262626262626"},"repository":{"context_snapshot_id":"rcs_91a9e83a09aa6dad3fe9ba5701aa1e7090330b8af07fad1b62e37a1fa3a7b2c7","head_oid":"sha256:1616161616161616161616161616161616161616161616161616161616161616","tree_oid":"sha256:1717171717171717171717171717171717171717171717171717171717171717","checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818"},"gate_ids":["gat_1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d","gat_1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e"],"prompt_digest":"2727272727272727272727272727272727272727272727272727272727272727","policy":{"policy_snapshot_digest":"0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b0b","policy_generation":2,"dogfood_binding_digest":"2828282828282828282828282828282828282828282828282828282828282828","tool_policy_digest":"0606060606060606060606060606060606060606060606060606060606060606","egress_policy_digest":"0505050505050505050505050505050505050505050505050505050505050505","containment_policy_digest":"2929292929292929292929292929292929292929292929292929292929292929"},"budget_reservation_id":"dbr_2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a","deadline_unix_ms":3000,"output_schema_digest":"2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b2b"},"intent_id":"dfi_c84d3bc08ac144e2b99c7537c332fa4d286c0e10719e374f1c428fe305fd5b23","launch_grant_id":"dfg_d95e65fb3c7b9ad82d88003b276f7fbbbb10b51036d246a42bf742a8787b3084","credential_projection_digest":"add2d0ddbd69262533f950dc6374b7f28917e39d8b7aa4ff64caf5df3d5fb9dd","budget_settlement":{"schema_version":"v1alpha1","reservation_id":"dbr_2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a2a","reservation_digest":"fed06f921628d0899f70cb12edf635da4dc9afe5a908192dd3ec6a6d98dd6814","settled_at_unix_ms":2300,"cost_micro_usd":{"knowledge":"known","used":100,"released":900,"overrun":0},"invocations":{"knowledge":"known","used":1,"released":0,"overrun":0},"wall_time_ms":{"knowledge":"known","used":100,"released":400,"overrun":0},"concurrency":{"knowledge":"known","used":1,"released":0,"overrun":0}},"repository_context_post_observation_digest":"7727bafb375edae8c83a65e607c126499a82b77c049f242e0dcfea942470b15e","provider_probe_observation_digest":"9ee793b9cb4124b834d783d3053986f5c90da8aaf093eb46c899aa398f5248ea","attestor_principal_id":"pri_3131313131313131313131313131313131313131313131313131313131313131","process":{"state":{"kind":"exited","code":0},"started_at_unix_ms":2000,"ended_at_unix_ms":2200,"observation_digest":"3232323232323232323232323232323232323232323232323232323232323232"},"artifacts":{"stdout":{"digest":"3333333333333333333333333333333333333333333333333333333333333333","size_bytes":10},"stderr":{"digest":"3434343434343434343434343434343434343434343434343434343434343434","size_bytes":11},"events":{"digest":"3535353535353535353535353535353535353535353535353535353535353535","size_bytes":12},"proxy":{"digest":"3636363636363636363636363636363636363636363636363636363636363636","size_bytes":13},"containment_receipt_digest":"3737373737373737373737373737373737373737373737373737373737373737","egress_receipt_digest":"3838383838383838383838383838383838383838383838383838383838383838","canary_observation_digest":"3939393939393939393939393939393939393939393939393939393939393939","process_tree_observation_digest":"3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a3a","artifact_manifest_digest":"3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b3b","retained_artifacts":[{"digest":"3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c3c","size_bytes":14},{"digest":"3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d3d","size_bytes":15}],"retained_artifact_count":2,"retained_artifact_size_bytes":29},"proposal":{"kind":"validated","proposal_id":"cnt_3030303030303030303030303030303030303030303030303030303030303030","proposal_digest":"838c6d578f8ac3fe2a429587b79426400e4697e45c31d1c682ff10bdfd1833d4","artifact":{"digest":"110513d7d9932595447022880ed45e8d1b90a03d939957596fdbb0dcf37d9212","size_bytes":745}},"cleanup":{"kind":"proved_empty","receipt_digest":"3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e3e","observed_at_unix_ms":2400},"attested_at_unix_ms":2500}}"#,
);
const GOLDEN_PROPOSAL: &str = r#"{"base_checkpoint_digest":"1919191919191919191919191919191919191919191919191919191919191919","base_checkpoint_id":"ckp_1818181818181818181818181818181818181818181818181818181818181818","gate_ids":["gat_1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d1d","gat_1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e1e"],"operations":[{"mutation":{"content_utf8":"fn main() {}\n","kind":"write"},"path":"src/file-000.rs","preimage":{"digest":"3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f3f","kind":"digest"}}],"producing_attempt_id":"atm_1414141414141414141414141414141414141414141414141414141414141414","proposal_id":"cnt_3030303030303030303030303030303030303030303030303030303030303030","schema_version":1}"#;

#[derive(Clone, Deserialize)]
struct Fixture {
    passport: ProviderRuntimePassportV1,
    enrollment: ProviderEnrollmentClaimsV2,
    intent: DogfoodReadOnlyIntentV1,
    grant: DogfoodLaunchGrantClaimsV1,
    projection: ProviderCredentialProjectionV1,
    reservation: DogfoodBudgetReservationV1,
    context: RepositoryContextSnapshotV1,
    post: RepositoryContextPostObservationV1,
    probe: ProviderProbeObservationV1,
    endpoint: ProviderEndpointObservationV1,
    version: ProviderVersionObservationV1,
    profile: ProviderProfileObservationV1,
    proposal: Option<PatchProposal>,
    run: DogfoodRunV1,
}

fn digest(seed: u8) -> Blake3Digest {
    Blake3Digest::from_bytes([seed; 32])
}

fn runtime(provider: LaunchProvider) -> (&'static str, &'static str) {
    match provider {
        LaunchProvider::Claude => ("2.1.251", "bin/claude"),
        LaunchProvider::Codex => ("0.150.1", "bin/codex"),
        LaunchProvider::Cursor => ("2026.08.11", "bin/cursor-agent"),
        LaunchProvider::Agy => ("1.1.19", "bin/agy"),
    }
}

fn passport(provider: LaunchProvider) -> ProviderRuntimePassportV1 {
    let (version, entrypoint) = runtime(provider);
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

fn fixture(provider: LaunchProvider) -> Fixture {
    let json = decode_unique_value(BASE_FIXTURE.as_bytes()).unwrap();
    let mut value: Fixture = serde_json::from_value(json).unwrap();
    if provider != LaunchProvider::Claude {
        rebind_provider(&mut value, provider);
    }
    value
}

fn rebind_provider(value: &mut Fixture, provider: LaunchProvider) {
    value.passport = passport(provider);
    let protocol = value.passport.protocol;
    value.enrollment.provider = provider;
    value.enrollment.protocol = protocol;
    value.enrollment.runtime_passport_id = value.passport.passport_id().unwrap();
    value.enrollment.runtime_version = value.passport.version.clone();

    value.probe.subject.provider = provider;
    value.probe.subject.protocol = protocol;
    value.probe.subject.runtime_passport_id = value.enrollment.runtime_passport_id.clone();
    let probe_digest = value.probe.digest().unwrap();
    for subject in [
        &mut value.endpoint.subject,
        &mut value.version.subject,
        &mut value.profile.subject,
    ] {
        *subject = value.probe.subject.clone();
    }
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
}

impl Fixture {
    fn verify(&self) -> Result<(), WireError> {
        verify_dogfood_run_binding(
            &self.run,
            &DogfoodRunBindingSubjects {
                grant: &self.grant,
                intent: &self.intent,
                enrollment: &self.enrollment,
                passport: &self.passport,
                projection: &self.projection,
                reservation: &self.reservation,
                context_snapshot: &self.context,
                post_context: &self.post,
                probe: &self.probe,
                endpoint: &self.endpoint,
                version: &self.version,
                profile: &self.profile,
                proposal: self.proposal.as_ref(),
            },
        )
    }
}

fn refusal(value: &Fixture, code: &'static str) {
    let error = value.verify().unwrap_err();
    assert_eq!(error.code(), code, "{error}");
}

fn rejects(code: &'static str, mutate: impl FnOnce(&mut Fixture)) {
    let mut value = fixture(LaunchProvider::Claude);
    mutate(&mut value);
    refusal(&value, code);
}

fn bad(code: &'static str, mutate: impl FnOnce(&mut Fixture)) {
    let mut value = fixture(LaunchProvider::Claude);
    value.run.intent_id = DogfoodIntentId::from_digest(digest(200));
    mutate(&mut value);
    refusal(&value, code);
}

fn write_operation(index: usize, content_utf8: String) -> PatchOperation {
    PatchOperation {
        path: serde_json::from_value(serde_json::Value::String(format!("src/file-{index:03}.rs")))
            .unwrap(),
        preimage: Preimage::Digest { digest: digest(63) },
        mutation: PatchMutation::Write { content_utf8 },
    }
}

fn bind_proposal(value: &mut Fixture) {
    let proposal = value.proposal.as_ref().unwrap();
    let bytes = canonical_json(proposal).unwrap();
    value.run.proposal = DogfoodProposalObservationV1::Validated {
        proposal_id: proposal.proposal_id.clone(),
        proposal_digest: hash_framed_bytes(DOGFOOD_PATCH_PROPOSAL_DIGEST_DOMAIN, &bytes).unwrap(),
        artifact: DogfoodArtifactRefV1 {
            digest: hash_framed_bytes(DOGFOOD_PATCH_PROPOSAL_ARTIFACT_DIGEST_DOMAIN, &bytes)
                .unwrap(),
            size_bytes: bytes.len() as u64,
        },
    };
}
#[test]
fn all_providers_and_literal_proposal_hashes_close() {
    for provider in PROVIDERS {
        fixture(provider).verify().unwrap();
    }
    let value = fixture(LaunchProvider::Claude);
    let bytes = canonical_json(value.proposal.as_ref().unwrap()).unwrap();
    assert_eq!(bytes, GOLDEN_PROPOSAL.as_bytes());
    assert_eq!(bytes.len(), 745);
    assert_eq!(
        [
            DOGFOOD_PATCH_PROPOSAL_DIGEST_DOMAIN,
            DOGFOOD_PATCH_PROPOSAL_ARTIFACT_DIGEST_DOMAIN,
        ],
        [
            "dogfood.patch-proposal.v1alpha1",
            "dogfood.patch-proposal-artifact.v1alpha1",
        ]
    );
    let hash = |domain| hash_framed_bytes(domain, &bytes).unwrap().to_hex();
    assert_eq!(
        [
            hash(DOGFOOD_PATCH_PROPOSAL_DIGEST_DOMAIN),
            hash(DOGFOOD_PATCH_PROPOSAL_ARTIFACT_DIGEST_DOMAIN),
        ],
        [
            "838c6d578f8ac3fe2a429587b79426400e4697e45c31d1c682ff10bdfd1833d4",
            "110513d7d9932595447022880ed45e8d1b90a03d939957596fdbb0dcf37d9212",
        ]
    );
}

#[rustfmt::skip]
#[test]
fn malformed_bodies_keep_their_original_precedence() {
    bad("DOGFOOD_RUN_INVALID", |v| v.run.schema_version.clear());
    bad("DOGFOOD_INTENT_INVALID", |v| v.intent.schema_version.clear());
    bad("DOGFOOD_GRANT_INVALID", |v| v.grant.schema_version.clear());
    bad("PROVIDER_ENROLLMENT_INVALID", |v| v.enrollment.schema_version.clear());
    bad("RUNTIME_PASSPORT_MALFORMED", |v| v.passport.deployment_root = "/tmp/runtime".into());
    bad("CREDENTIAL_PROJECTION_INVALID", |v| v.projection.schema_version.clear());
    bad("DOGFOOD_BUDGET_RESERVATION_INVALID", |v| v.reservation.schema_version.clear());
    bad("REPOSITORY_CONTEXT_INVALID", |v| v.context.schema_version.clear());
    bad("REPOSITORY_CONTEXT_INVALID", |v| v.post.schema_version.clear());
    bad("PROVIDER_PROBE_OBSERVATION_INVALID", |v| v.probe.schema_version.clear());
    bad("PROVIDER_ENDPOINT_OBSERVATION_INVALID", |v| v.endpoint.schema_version.clear());
    bad("PROVIDER_VERSION_OBSERVATION_INVALID", |v| v.version.schema_version.clear());
    bad("PROVIDER_PROFILE_OBSERVATION_INVALID", |v| v.profile.schema_version.clear());
    bad("UNSUPPORTED_SCHEMA", |v| v.proposal.as_mut().unwrap().schema_version = 0);
}

#[test]
fn substitutions_across_w0_through_w5_refuse() {
    rejects("DOGFOOD_GRANT_SUBJECT_MISMATCH", |v| {
        v.grant.request_digest = digest(201);
    });
    rejects("PROVIDER_ENROLLMENT_SUBJECT_MISMATCH", |v| {
        v.enrollment.service_identity_id = PrincipalId::from_digest(digest(202));
    });
    rejects("RUNTIME_PASSPORT_ID_MISMATCH", |v| {
        v.passport.version = "changed".into();
        v.passport.deployment_root = "/usr/lib/bullet/providers/claude/changed".into();
    });
    rejects("DOGFOOD_BUDGET_RESERVATION_ID_MISMATCH", |v| {
        v.reservation.reservation_id = DogfoodBudgetReservationId::from_digest(digest(203));
    });
    rejects("DOGFOOD_BUDGET_SUBJECT_MISMATCH", |v| {
        v.reservation.budget_policy_digest = digest(207);
    });
    rejects("REPOSITORY_CONTEXT_ID_MISMATCH", |v| {
        v.context.scope_grant_digest = digest(208);
    });
    rejects("REPOSITORY_CONTEXT_POST_MISMATCH", |v| {
        v.post.observed_checkpoint_digest = digest(209);
    });
    rejects("PROVIDER_OBSERVATION_SUBJECT_MISMATCH", |v| {
        v.profile.subject.service_identity_id = PrincipalId::from_digest(digest(210));
    });
    rejects("PROVIDER_PROBE_OBSERVATION_MISMATCH", |v| {
        v.probe.probe_grant_digest = digest(211);
    });
    rejects("PROVIDER_ENDPOINT_OBSERVATION_MISMATCH", |v| {
        v.endpoint.entrypoint_blake3 = digest(212);
    });
    rejects("PROVIDER_VERSION_OBSERVATION_MISMATCH", |v| {
        v.version.runtime_version = "changed".into();
    });
    rejects("PROVIDER_PROFILE_OBSERVATION_MISMATCH", |v| {
        v.profile.effective_identity_artifact_digest = digest(213);
    });
    rejects("DOGFOOD_RUN_SUBJECT_MISMATCH", |v| {
        v.run.subject.prompt_digest = digest(214);
    });
    rejects("DOGFOOD_RUN_SUBJECT_MISMATCH", |v| {
        v.run.intent_id = DogfoodIntentId::from_digest(digest(215));
    });
    rejects("DOGFOOD_RUN_SUBJECT_MISMATCH", |v| {
        v.run.launch_grant_id = DogfoodGrantId::from_digest(digest(216));
    });
    rejects("DOGFOOD_RUN_RESOURCE_MISMATCH", |v| {
        v.projection.service_identity_id = PrincipalId::from_digest(digest(220));
    });
    rejects("DOGFOOD_RUN_RESOURCE_MISMATCH", |v| {
        v.projection.target_policy_digest = digest(221);
    });
    rejects("DOGFOOD_RUN_RESOURCE_MISMATCH", |v| {
        v.run.repository_context_post_observation_digest = digest(222);
    });
    rejects("DOGFOOD_RUN_RESOURCE_MISMATCH", |v| {
        v.run.provider_probe_observation_digest = digest(223);
    });
}

fn set_times(value: &mut Fixture, start: Option<u64>, end: Option<u64>, times: [u64; 4]) {
    value.run.process.started_at_unix_ms = start;
    value.run.process.ended_at_unix_ms = end;
    value.post.observed_at_unix_ms = times[0];
    value.run.repository_context_post_observation_digest = value.post.observation_digest().unwrap();
    value.run.budget_settlement.settled_at_unix_ms = times[1];
    value.run.cleanup = DogfoodCleanupObservationV1::ProvedEmpty {
        receipt_digest: digest(62),
        observed_at_unix_ms: times[2],
    };
    value.run.attested_at_unix_ms = times[3];
}

fn make_unknown(value: &mut Fixture, start: Option<u64>, end: Option<u64>, terminal: u64) {
    value.run.process.state = DogfoodProcessStateV1::OutcomeUnknown;
    value.proposal = None;
    value.run.proposal = DogfoodProposalObservationV1::Absent;
    value.run.budget_settlement.cost_micro_usd =
        DogfoodUsageSettlementV1::Unknown { retained: 1_000 };
    value.run.budget_settlement.invocations = DogfoodUsageSettlementV1::Unknown { retained: 1 };
    value.run.budget_settlement.wall_time_ms = DogfoodUsageSettlementV1::Unknown { retained: 500 };
    value.run.budget_settlement.concurrency = DogfoodUsageSettlementV1::Unknown { retained: 1 };
    set_times(value, start, end, [terminal; 4]);
}

#[test]
fn half_open_deadline_causal_and_unknown_edges_are_exact() {
    let mut value = fixture(LaunchProvider::Claude);
    value.verify().unwrap();
    set_times(
        &mut value,
        Some(1_999),
        Some(2_200),
        [2_200, 2_300, 2_400, 2_500],
    );
    refusal(&value, "DOGFOOD_RUN_TIME_MISMATCH");
    set_times(&mut value, Some(2_800), Some(2_800), [2_800; 4]);
    refusal(&value, "DOGFOOD_RUN_TIME_MISMATCH");
    set_times(&mut value, Some(2_000), Some(3_000), [3_000; 4]);
    value.verify().unwrap();
    set_times(&mut value, Some(2_000), Some(3_001), [3_001; 4]);
    refusal(&value, "DOGFOOD_RUN_TIME_MISMATCH");

    value = fixture(LaunchProvider::Claude);
    value.projection.activates_at_unix_ms = 2_001;
    value.run.process.started_at_unix_ms = Some(2_001);
    value.run.credential_projection_digest = value.projection.projection_digest().unwrap();
    refusal(&value, "DOGFOOD_RUN_TIME_MISMATCH");
    value = fixture(LaunchProvider::Claude);
    value.projection.expires_at_unix_ms = 2_799;
    value.run.credential_projection_digest = value.projection.projection_digest().unwrap();
    refusal(&value, "DOGFOOD_RUN_TIME_MISMATCH");
    value = fixture(LaunchProvider::Claude);
    value.reservation.reserved_at_unix_ms = 2_001;
    value.run.budget_settlement.reservation_digest =
        value.reservation.reservation_digest().unwrap();
    refusal(&value, "DOGFOOD_RUN_TIME_MISMATCH");
    for times in [
        [2_199, 2_300, 2_400, 2_500],
        [2_250, 2_249, 2_400, 2_500],
        [2_250, 2_300, 2_299, 2_500],
        [2_250, 2_300, 2_400, 2_399],
    ] {
        let mut causal = fixture(LaunchProvider::Claude);
        set_times(&mut causal, Some(2_000), Some(2_200), times);
        refusal(&causal, "DOGFOOD_RUN_TIME_MISMATCH");
    }
    let mut causal = fixture(LaunchProvider::Claude);
    set_times(&mut causal, Some(2_000), Some(2_200), [2_200; 4]);
    causal.verify().unwrap();

    for (start, end, terminal) in [
        (None, None, 2_500),
        (Some(2_000), None, 2_500),
        (None, Some(3_000), 3_000),
    ] {
        let mut unknown = fixture(LaunchProvider::Claude);
        make_unknown(&mut unknown, start, end, terminal);
        unknown.verify().unwrap();
    }
    let mut unknown = fixture(LaunchProvider::Claude);
    make_unknown(&mut unknown, Some(2_800), None, 2_800);
    refusal(&unknown, "DOGFOOD_RUN_TIME_MISMATCH");
    make_unknown(&mut unknown, None, Some(1_999), 2_500);
    refusal(&unknown, "DOGFOOD_RUN_TIME_MISMATCH");
    make_unknown(&mut unknown, None, Some(3_001), 3_001);
    refusal(&unknown, "DOGFOOD_RUN_TIME_MISMATCH");
    unknown.run.budget_settlement.cost_micro_usd = DogfoodUsageSettlementV1::Known {
        used: 0,
        released: 1_000,
        overrun: 0,
    };
    refusal(&unknown, "DOGFOOD_RUN_PROCESS_MISMATCH");
}

fn proposal_rejects(mutate: impl FnOnce(&mut Fixture)) {
    rejects("DOGFOOD_RUN_PROPOSAL_MISMATCH", mutate);
}
fn proposal(value: &mut Fixture) -> &mut PatchProposal {
    value.proposal.as_mut().unwrap()
}
fn observed(value: &mut Fixture) -> (&mut ContentId, &mut Blake3Digest, &mut DogfoodArtifactRefV1) {
    let DogfoodProposalObservationV1::Validated {
        proposal_id,
        proposal_digest,
        artifact,
    } = &mut value.run.proposal
    else {
        unreachable!()
    };
    (proposal_id, proposal_digest, artifact)
}
#[test]
fn proposal_pairing_and_every_exact_binding_refuse_tampering() {
    for observation in [
        DogfoodProposalObservationV1::Absent,
        DogfoodProposalObservationV1::Rejected {
            artifact: DogfoodArtifactRefV1 {
                digest: digest(80),
                size_bytes: 1,
            },
        },
    ] {
        let mut value = fixture(LaunchProvider::Claude);
        value.proposal = None;
        value.run.proposal = observation;
        value.verify().unwrap();
        value.proposal = fixture(LaunchProvider::Claude).proposal;
        refusal(&value, "DOGFOOD_RUN_PROPOSAL_MISMATCH");
    }
    proposal_rejects(|v| v.proposal = None);

    proposal_rejects(|v| proposal(v).proposal_id = ContentId::from_digest(digest(81)));
    proposal_rejects(|v| {
        proposal(v).producing_attempt_id = AttemptId::from_digest(digest(82));
    });
    proposal_rejects(|v| {
        proposal(v).base_checkpoint_id = CheckpointId::from_digest(digest(83));
    });
    proposal_rejects(|v| proposal(v).base_checkpoint_digest = digest(84));
    proposal_rejects(|v| proposal(v).gate_ids.swap(0, 1));
    proposal_rejects(|v| {
        let PatchMutation::Write { content_utf8 } = &mut proposal(v).operations[0].mutation else {
            unreachable!()
        };
        content_utf8.push('x');
    });
    proposal_rejects(|v| *observed(v).0 = ContentId::from_digest(digest(85)));
    proposal_rejects(|v| *observed(v).1 = digest(86));
    proposal_rejects(|v| observed(v).2.digest = digest(87));
    proposal_rejects(|v| observed(v).2.size_bytes += 1);
}

fn sized_operations(total: usize) -> Vec<PatchOperation> {
    let mut remaining = total;
    let mut operations = Vec::new();
    while remaining > 0 {
        let size = remaining.min(DEFAULT_MAX_CONTENT_BYTES);
        operations.push(write_operation(operations.len(), "x".repeat(size)));
        remaining -= size;
    }
    operations
}

#[test]
fn operation_and_aggregate_content_limits_are_inclusive() {
    let mut value = fixture(LaunchProvider::Claude);
    value.proposal.as_mut().unwrap().operations = (0..128)
        .map(|index| write_operation(index, "x".into()))
        .collect();
    bind_proposal(&mut value);
    value.verify().unwrap();
    value
        .proposal
        .as_mut()
        .unwrap()
        .operations
        .push(write_operation(128, "x".into()));
    bind_proposal(&mut value);
    refusal(&value, "DOGFOOD_RUN_PROPOSAL_MISMATCH");

    value = fixture(LaunchProvider::Claude);
    value.proposal.as_mut().unwrap().operations = sized_operations(MAX_DOGFOOD_PATCH_CONTENT_BYTES);
    bind_proposal(&mut value);
    value.verify().unwrap();
    value.proposal.as_mut().unwrap().operations =
        sized_operations(MAX_DOGFOOD_PATCH_CONTENT_BYTES + 1);
    bind_proposal(&mut value);
    refusal(&value, "DOGFOOD_RUN_PROPOSAL_MISMATCH");
}
