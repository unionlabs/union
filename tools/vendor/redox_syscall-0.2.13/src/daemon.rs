use core::convert::Infallible;

use super::{
    clone,
    CloneFlags,
    close,
    EIO,
    Error,
    exit,
    pipe2,
    read,
    Result,
    write,
};

#[must_use = "Daemon::ready must be called"]
pub struct Daemon {
    write_pipe: usize,
}

impl Daemon {
    pub fn new<F: FnOnce(Daemon) -> Infallible>(f: F) -> Result<Infallible> {
        let mut pipes = [0; 2];
        pipe2(&mut pipes, 0)?;

        let [read_pipe, write_pipe] = pipes;

        if unsafe { clone(CloneFlags::empty())? } == 0 {
            let _ = close(read_pipe);

            f(Daemon {
                write_pipe,
            });
            // TODO: Replace Infallible with the never type once it is stabilized.
            unreachable!();
        } else {
            let _ = close(write_pipe);

            let mut data = [0];
            let res = read(read_pipe, &mut data);
            let _ = close(read_pipe);

            if res? == 1 {
                exit(data[0] as usize)?;
                unreachable!();
            } else {
                Err(Error::new(EIO))
            }
        }
    }

    pub fn ready(self) -> Result<()> {
        let res = write(self.write_pipe, &[0]);
        let _ = close(self.write_pipe);

        if res? == 1 {
            Ok(())
        } else {
            Err(Error::new(EIO))
        }
    }
}
