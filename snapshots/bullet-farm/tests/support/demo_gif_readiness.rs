//! Test-only fork handshake; the actual recorder clock, loop and cleanup run unchanged.
use std::{
    path::Path,
    process::{Command, Output},
};

pub fn capture(root: &Path, recorder: &Path) -> Output {
    let source = r#"
import os,runpy,select,signal,sys
module=runpy.run_path(sys.argv[1])
reader,writer=os.pipe()
os.set_inheritable(writer,True)
native_fork=module['pty'].fork

def ready_fork():
    pid,fd=native_fork()
    if pid==0:
        os.close(reader)
        return pid,fd
    os.close(writer)
    try:
        assert select.select([reader],[],[],2)[0], 'child signal setup not acknowledged'
        assert os.read(reader,6)==b'ready\n', 'invalid child readiness'
    except BaseException:
        status,gone=module['finish_group'](pid)
        os.close(fd)
        assert status is not None and gone, 'readiness failure left process custody unresolved'
        raise
    finally:
        os.close(reader)
    return pid,fd

# The real PTY child still executes the real recorder's requested argv. Only
# the parent fork return is gated, so no timeout signal can beat signal setup.
module['pty'].fork=ready_fork
child=("import os,signal,time;signal.alarm(5);"
       "signal.signal(signal.SIGTERM,signal.SIG_IGN);"
       "signal.signal(signal.SIGHUP,signal.SIG_IGN);"
       f"os.write({writer},b'ready\\n');os.close({writer});"
       "print('ready',flush=True);time.sleep(10)")
sys.argv=[sys.argv[1],'--cast',sys.argv[2],'--transcript',sys.argv[3],
          '--max-seconds','0.2','--','python3','-c',child]
sys.exit(module['main']())
"#;
    Command::new("timeout")
        .args(["--kill-after=1s", "7s", "python3", "-c", source])
        .arg(recorder)
        .arg(root.join("session.cast"))
        .arg(root.join("transcript.txt"))
        .output()
        .expect("readiness-gated real recorder must execute")
}
