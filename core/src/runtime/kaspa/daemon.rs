use crate::imports::*;
use crate::runtime::kaspa::{Config, KaspadServiceEvents};
// use alloc::task;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;
use workflow_core::prelude::DuplexChannel;

struct Inner {
    path: Option<PathBuf>,
    is_running: Arc<AtomicBool>,
    pid: Mutex<Option<u32>>,
    service_events: Channel<KaspadServiceEvents>,
    task_ctl: DuplexChannel,
}

#[derive(Clone)]
pub struct Daemon {
    inner: Arc<Inner>,
}

impl Daemon {
    pub fn new(path: Option<PathBuf>, service_events: &Channel<KaspadServiceEvents>) -> Self {
        Self {
            inner: Arc::new(Inner {
                path,
                is_running: Arc::new(AtomicBool::new(false)),
                pid: Mutex::new(None),
                service_events: (*service_events).clone(),
                task_ctl: DuplexChannel::oneshot(),
            }),
        }
    }

    fn inner(&self) -> &Inner {
        &self.inner
    }

    fn is_running(&self) -> bool {
        self.inner().is_running.load(Ordering::SeqCst)
    }
}

#[async_trait]
impl super::Kaspad for Daemon {
    async fn start(&self, config: Config) -> Result<()> {
        let mut cmd = if let Some(path) = self.inner().path.clone() {
            Command::new(path)
        } else {
            let path = std::env::current_exe()?;
            Command::new(path)
        };

        let cmd = cmd
            .args(config)
            .env("KASPA_NG_NODE", "1")
            .stdout(Stdio::piped());

        let is_running = self.inner().is_running.clone();
        is_running.store(true, Ordering::SeqCst);
        let mut child = cmd.spawn().map_err(Error::NodeStartupError)?;
        let stdout = child.stdout.take().ok_or(Error::NodeStdoutHandleError)?;
        *self.inner.pid.lock().unwrap() = child.id();

        let mut reader = BufReader::new(stdout).lines();
        let stdout_relay_sender = self.inner.service_events.sender.clone();
        let task_ctl = self.inner.task_ctl.clone();
        // let task_ctl_resop = self.inner.task_ctl.request.clone();
        tokio::spawn(async move {
            loop {
                // println!("loop...");
                select! {
                    _ = task_ctl.request.recv().fuse() => {
                        println!("task_ctl.recv()...");
                        println!("********************************************************* DAEMON TERMINATION");

                        if let Err(err) = child.start_kill() {
                            println!("child start_kill error: {:?}", err);
                        }

                        // break;
                    }
                    status = child.wait().fuse() => {
                        match status {
                            Ok(status) => {
                                println!("child status was: {:?}", status);
                            }
                            Err(err) => {
                                println!("child error was: {:?}", err);
                            }
                        }
                        println!("********************************************************* DAEMON CHILD STATUS");

                        is_running.store(false,Ordering::SeqCst);

                        break;
                    }

                    line = reader.next_line().fuse() => {
                        match line {
                            Ok(Some(line)) => {
                                // println!("Child Daemon Stdout: {}", text);
                                stdout_relay_sender.send(KaspadServiceEvents::Stdout { line }).await.unwrap();
                            },
                            v => {
                                println!("Node daemon stdout error: {:?}", v);
                                // break;
                            }
                        }
                    }
                }
            }

            task_ctl.response.send(()).await.unwrap();
        });

        Ok(())
    }

    async fn stop(&self) -> Result<()> {
        println!("********************************************************* DAEMON STOP [1]");
        if self.is_running() {
            println!("********************************************************* DAEMON SIGNALING STOP [2]");
            self.inner.task_ctl.signal(()).await?;
        }
        println!(
            "********************************************************* DAEMON SIGNALING STOP [3]"
        );
        Ok(())
    }
}