use super::*;

#[cfg(test)]
use std::cell::RefCell;
#[cfg(all(test, target_os = "linux"))]
use std::path::PathBuf;

#[cfg(all(test, target_os = "linux"))]
thread_local! {
    static REPLACE_CURRENT_DURING_GENESIS_AUTHORITY: RefCell<Option<PathBuf>> = const {
        RefCell::new(None)
    };
    static SWAP_GENERATION_AFTER_APPEND: RefCell<Option<(PathBuf, PathBuf)>> = const {
        RefCell::new(None)
    };
    static SWAP_SUBJECT_BEFORE_RETURN: RefCell<Option<(PathBuf, PathBuf)>> = const {
        RefCell::new(None)
    };
}

#[cfg(test)]
thread_local! {
    static BEFORE_SUBJECT_GUARD: RefCell<Option<Box<dyn FnOnce()>>> = const {
        RefCell::new(None)
    };
}

#[cfg(all(test, target_os = "linux"))]
pub(super) fn test_replace_current_during_genesis_authority(path: PathBuf) {
    REPLACE_CURRENT_DURING_GENESIS_AUTHORITY.with(|selected| {
        assert!(selected.borrow_mut().replace(path).is_none());
    });
}

#[cfg(all(test, target_os = "linux"))]
pub(super) fn test_swap_generation_after_append(canonical: PathBuf, replacement: PathBuf) {
    SWAP_GENERATION_AFTER_APPEND.with(|selected| {
        assert!(
            selected
                .borrow_mut()
                .replace((canonical, replacement))
                .is_none()
        );
    });
}

#[cfg(all(test, target_os = "linux"))]
pub(super) fn test_swap_subject_before_return(canonical: PathBuf, replacement: PathBuf) {
    SWAP_SUBJECT_BEFORE_RETURN.with(|selected| {
        assert!(
            selected
                .borrow_mut()
                .replace((canonical, replacement))
                .is_none()
        );
    });
}

impl Ledger {
    #[cfg(test)]
    pub(in crate::coord::store) fn test_before_subject_guard(hook: impl FnOnce() + 'static) {
        BEFORE_SUBJECT_GUARD.with(|selected| {
            assert!(selected.borrow_mut().replace(Box::new(hook)).is_none());
        });
    }

    #[cfg(test)]
    pub(in crate::coord::store) fn transact<F>(
        &self,
        expected_generation_id: &str,
        request_id: &str,
        make_record: F,
    ) -> Result<RequestTransaction, CoordError>
    where
        F: FnOnce(&LedgerView) -> Result<Record, CoordError>,
    {
        self.transact_guarded(
            expected_generation_id,
            request_id,
            |view| Ok((make_record(view)?, ())),
            |()| Ok(()),
        )
    }

    pub(in crate::coord::store) fn transact_guarded<F, V, G>(
        &self,
        expected_generation_id: &str,
        request_id: &str,
        make_record: F,
        validate_guard: V,
    ) -> Result<RequestTransaction, CoordError>
    where
        F: FnOnce(&LedgerView) -> Result<(Record, G), CoordError>,
        V: FnOnce(&G) -> Result<(), CoordError>,
    {
        self.transact_loaded_guarded(
            expected_generation_id,
            request_id,
            |loaded| make_record(&loaded.view),
            validate_guard,
        )
    }

    pub(super) fn transact_loaded<F>(
        &self,
        expected_generation_id: &str,
        request_id: &str,
        make_record: F,
    ) -> Result<RequestTransaction, CoordError>
    where
        F: FnOnce(&mut Loaded) -> Result<Record, CoordError>,
    {
        self.transact_loaded_guarded(
            expected_generation_id,
            request_id,
            |loaded| Ok((make_record(loaded)?, ())),
            |()| Ok(()),
        )
    }

    fn transact_loaded_guarded<F, V, G>(
        &self,
        expected_generation_id: &str,
        request_id: &str,
        make_record: F,
        validate_guard: V,
    ) -> Result<RequestTransaction, CoordError>
    where
        F: FnOnce(&mut Loaded) -> Result<(Record, G), CoordError>,
        V: FnOnce(&G) -> Result<(), CoordError>,
    {
        validate_request_id(request_id)?;
        let probe = fs::probe(&self.coord_dir)?;
        let lock = match probe.presence() {
            fs::Presence::Absent => return Err(uninitialized()),
            fs::Presence::Legacy => return Err(recovery_required()),
            fs::Presence::Retired => return Err(recovery_in_progress()),
            fs::Presence::Current => probe.into_lock(&self.coord_dir, true)?,
        };
        let current = lock
            .current()?
            .ok_or_else(|| changed("CURRENT is absent under the stable LOCK"))?;
        if current.generation_id().as_str() != expected_generation_id {
            return Err(changed("append expected another generation"));
        }
        let mut loaded = self.load_locked(&lock, Some(&current), true)?;
        if let Some(existing) = loaded
            .inspection
            .entries
            .iter()
            .find(|entry| entry.request_id == request_id)
        {
            let record = existing.record.clone();
            let receipt =
                verify::receipt(loaded.manifest.generation_id().as_str(), &existing.receipt);
            let watermark = verify::request_watermark(&loaded.view.watermark, &receipt)?;
            let recovery_guard = match loaded.manifest.body {
                GenerationManifestBody::Genesis(_) => None,
                GenerationManifestBody::RecoveryBaseline(_) => Some(
                    crate::coord::generation::recovery::verify_published_recovery(
                        lock.root(),
                        &loaded.manifest,
                    )?,
                ),
            };
            self.revalidate_before_return(&mut loaded, None, &lock, recovery_guard.as_ref())?;
            return Ok(RequestTransaction {
                existing: true,
                record,
                receipt,
                watermark,
                view: loaded.view,
            });
        }
        let (record, guard) = make_record(&mut loaded)?;
        {
            let request = AppendRequest {
                generation_id: loaded.manifest.generation_id().as_str(),
                sequence: loaded.inspection.position.next_sequence,
                previous_digest: &loaded.inspection.position.previous_digest,
                request_id,
                record: &record,
            };
            segment::validate_append_request(&request, &loaded.genesis_digest)?;
        }
        let recovery_guard = self.revalidate_before_append(&mut loaded, &lock)?;
        inject_before_subject_guard();
        validate_guard(&guard)?;
        let request = AppendRequest {
            generation_id: loaded.manifest.generation_id().as_str(),
            sequence: loaded.inspection.position.next_sequence,
            previous_digest: &loaded.inspection.position.previous_digest,
            request_id,
            record: &record,
        };
        let appended = segment::append_files(
            &mut loaded.files.segment,
            &loaded.files.pending,
            &request,
            &loaded.genesis_digest,
        )?;
        loaded.files.revalidate(&lock, true)?;
        inject_generation_swap()?;
        let expected_pointer = loaded.pointer.clone();
        let appended_files = loaded.files;
        let mut loaded = self.load_locked(&lock, Some(&expected_pointer), false)?;
        appended_files.revalidate(&lock, true)?;
        let durable = loaded
            .inspection
            .entries
            .iter()
            .find(|entry| entry.request_id == request_id)
            .ok_or_else(|| changed("reloaded generation omits the appended request"))?;
        if durable.receipt != appended
            || bullet_wire::canonical_json(&durable.record).map_err(wire)?
                != bullet_wire::canonical_json(&record).map_err(wire)?
        {
            return Err(changed(
                "reloaded request record or receipt differs from the append subject",
            ));
        }
        let record = durable.record.clone();
        let receipt = verify::receipt(loaded.manifest.generation_id().as_str(), &durable.receipt);
        let watermark = verify::request_watermark(&loaded.view.watermark, &receipt)?;
        if watermark != loaded.view.watermark {
            return Err(changed(
                "reloaded request is not the exact durable transaction watermark",
            ));
        }
        self.revalidate_before_return(
            &mut loaded,
            Some(&appended_files),
            &lock,
            recovery_guard.as_ref(),
        )?;
        Ok(RequestTransaction {
            existing: false,
            record,
            receipt,
            watermark,
            view: loaded.view,
        })
    }

    fn revalidate_before_append(
        &self,
        loaded: &mut Loaded,
        lock: &fs::CoordLock,
    ) -> Result<Option<crate::coord::generation::recovery::PublishedRecoveryGuard>, CoordError>
    {
        let recovery_guard = match loaded.manifest.body {
            GenerationManifestBody::Genesis(_) => None,
            GenerationManifestBody::RecoveryBaseline(_) => Some(
                crate::coord::generation::recovery::verify_published_recovery(
                    lock.root(),
                    &loaded.manifest,
                )?,
            ),
        };
        loaded.files.revalidate(lock, true)?;
        let manifest = loaded.files.load_manifest(loaded.pointer.generation_id())?;
        if manifest != loaded.manifest {
            return Err(changed("generation manifest changed before append"));
        }
        loaded.pointer.verify_manifest(&manifest)?;
        if let Some(guard) = recovery_guard.as_ref() {
            crate::coord::generation::recovery::reverify_published_recovery(
                lock.root(),
                &loaded.manifest,
                guard,
            )?;
        }
        if matches!(loaded.manifest.body, GenerationManifestBody::Genesis(_)) {
            replay_authority::verify_genesis(
                lock,
                &loaded.manifest,
                &loaded.pointer,
                inject_current_replacement,
            )?;
        }
        lock.revalidate()?;
        let current = lock
            .current()?
            .ok_or_else(|| changed("CURRENT disappeared before append"))?;
        if current != loaded.pointer {
            return Err(changed("CURRENT changed before append"));
        }
        match &loaded.manifest.body {
            GenerationManifestBody::Genesis(_) => {}
            GenerationManifestBody::RecoveryBaseline(_) => {
                recovery_guard
                    .as_ref()
                    .ok_or_else(|| changed("recovery guard is absent before append"))?
                    .revalidate()?;
            }
        }
        Ok(recovery_guard)
    }

    fn revalidate_before_return(
        &self,
        loaded: &mut Loaded,
        appended_files: Option<&fs::GenerationFiles>,
        lock: &fs::CoordLock,
        recovery_guard: Option<&crate::coord::generation::recovery::PublishedRecoveryGuard>,
    ) -> Result<(), CoordError> {
        match &loaded.manifest.body {
            GenerationManifestBody::Genesis(_) => {
                replay_authority::verify_genesis(lock, &loaded.manifest, &loaded.pointer, || {
                    Ok(())
                })?;
            }
            GenerationManifestBody::RecoveryBaseline(_) => {
                let guard = recovery_guard
                    .ok_or_else(|| changed("recovery guard is absent after append"))?;
                crate::coord::generation::recovery::reverify_published_recovery(
                    lock.root(),
                    &loaded.manifest,
                    guard,
                )?;
            }
        }
        inject_return_subject_swap()?;
        let manifest = loaded.files.load_manifest(loaded.pointer.generation_id())?;
        if manifest != loaded.manifest {
            return Err(changed("generation manifest changed after append replay"));
        }
        loaded.pointer.verify_manifest(&manifest)?;
        if let Some(files) = appended_files {
            files.revalidate(lock, true)?;
        }
        loaded.files.revalidate(lock, true)?;
        lock.revalidate()?;
        let current = lock
            .current()?
            .ok_or_else(|| changed("CURRENT disappeared after append replay"))?;
        if current != loaded.pointer {
            return Err(changed("CURRENT changed after append replay"));
        }
        if let Some(guard) = recovery_guard {
            guard.revalidate()?;
        }
        Ok(())
    }
}

#[cfg(test)]
fn inject_before_subject_guard() {
    BEFORE_SUBJECT_GUARD.with(|selected| {
        if let Some(hook) = selected.borrow_mut().take() {
            hook();
        }
    });
}

#[cfg(not(test))]
fn inject_before_subject_guard() {}

#[cfg(all(test, target_os = "linux"))]
fn inject_current_replacement() -> Result<(), CoordError> {
    use std::os::unix::fs::PermissionsExt;

    let Some(path) =
        REPLACE_CURRENT_DURING_GENESIS_AUTHORITY.with(|selected| selected.borrow_mut().take())
    else {
        return Ok(());
    };
    let bytes = std::fs::read(&path).map_err(CoordError::io)?;
    let displaced = path.with_file_name(".CURRENT.displaced-test");
    std::fs::rename(&path, displaced).map_err(CoordError::io)?;
    std::fs::write(&path, bytes).map_err(CoordError::io)?;
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o400)).map_err(CoordError::io)
}

#[cfg(all(test, target_os = "linux"))]
fn inject_generation_swap() -> Result<(), CoordError> {
    let Some((canonical, replacement)) =
        SWAP_GENERATION_AFTER_APPEND.with(|selected| selected.borrow_mut().take())
    else {
        return Ok(());
    };
    let displaced = canonical.with_file_name(".displaced-generation-test");
    std::fs::rename(&canonical, displaced).map_err(CoordError::io)?;
    std::fs::rename(replacement, canonical).map_err(CoordError::io)
}

#[cfg(all(test, target_os = "linux"))]
fn inject_return_subject_swap() -> Result<(), CoordError> {
    let Some((canonical, replacement)) =
        SWAP_SUBJECT_BEFORE_RETURN.with(|selected| selected.borrow_mut().take())
    else {
        return Ok(());
    };
    let displaced = canonical.with_file_name(".CURRENT.displaced-return-test");
    std::fs::rename(&canonical, displaced).map_err(CoordError::io)?;
    std::fs::rename(replacement, canonical).map_err(CoordError::io)
}

#[cfg(not(all(test, target_os = "linux")))]
fn inject_current_replacement() -> Result<(), CoordError> {
    Ok(())
}

#[cfg(not(all(test, target_os = "linux")))]
fn inject_generation_swap() -> Result<(), CoordError> {
    Ok(())
}

#[cfg(not(all(test, target_os = "linux")))]
fn inject_return_subject_swap() -> Result<(), CoordError> {
    Ok(())
}
