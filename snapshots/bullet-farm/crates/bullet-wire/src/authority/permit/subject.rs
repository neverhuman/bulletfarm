use super::{MutationPermitExpectation, MutationPermitSubject};

impl MutationPermitExpectation {
    #[must_use]
    pub fn subject(&self) -> MutationPermitSubject {
        MutationPermitSubject {
            audience: self.audience,
            operation: self.operation,
            authority_envelope_digest: self.authority_envelope_digest,
            authority_token_nonce: self.authority_token_nonce,
            mutation_id: self.mutation_id.clone(),
            reservation_id: self.reservation_id.clone(),
            request_digest: self.request_digest,
            repository_id: self.repository_id.clone(),
            workspace_id: self.workspace_id.clone(),
            workspace_generation: self.workspace_generation,
            attempt_id: self.attempt_id.clone(),
            attempt_fence: self.attempt_fence,
            authority_epoch: self.authority_epoch,
            freeze_generation: self.freeze_generation,
        }
    }
}
