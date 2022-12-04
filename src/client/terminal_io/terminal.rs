use futures::lock::Mutex;
use std::sync::Arc;
use tokio::sync::mpsc::UnboundedReceiver;
use tokio::task::{spawn, JoinHandle};

use super::output::TerminalOut;
use super::terminal_event::TerminalEvent;

pub struct Terminal {}

impl Terminal {
    pub async fn spawn_renderer(
        terminal_out: Arc<Mutex<Box<dyn TerminalOut + Send>>>,
        mut out_rx: UnboundedReceiver<TerminalEvent>,
    ) -> JoinHandle<()> {
        spawn(async move {
            loop {
                match out_rx.recv().await {
                    Some(TerminalEvent::View(state)) => {
                        if let Err(error) = terminal_out.lock().await.render(state) {
                            // TODO we should probably only restore the terminal here and propagate
                            // the error so that the actual process::exit happens inside our main
                            // function
                            return terminal_out
                                .lock()
                                .await
                                .quit(Some(&format!("ERROR: {}", error)), true);
                        }
                    }
                    Some(TerminalEvent::Quit) => {
                        return terminal_out.lock().await.quit(Some("Cya!"), false);
                    }
                    None => continue,
                }
            }
        })
    }
}