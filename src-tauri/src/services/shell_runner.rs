use std::io::{BufRead, BufReader, Read};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::model::CommandOutput;

fn append_line(buffer: &mut String, text: &str) {
    if !buffer.is_empty() {
        buffer.push('\n');
    }
    buffer.push_str(text);
}

fn spawn_reader<R>(
    reader: R,
    label: &str,
) -> thread::JoinHandle<Result<String, String>>
where
    R: Read + Send + 'static,
{
    let stream = label.to_string();
    thread::spawn(move || {
        let mut buffered = BufReader::new(reader);
        let mut line = String::new();
        let mut collected = String::new();
        loop {
            line.clear();
            match buffered.read_line(&mut line) {
                Ok(0) => break,
                Ok(_) => {
                    let text = line
                        .trim_end_matches('\n')
                        .trim_end_matches('\r')
                        .to_string();
                    append_line(&mut collected, &text);
                }
                Err(error) => {
                    return Err(format!("failed to read {stream}: {error}"));
                }
            }
        }
        Ok(collected)
    })
}

pub fn run_shell_command(
    command: &str,
    timeout_seconds: u64,
) -> Result<CommandOutput, String> {
    let started = Instant::now();
    let mut child = Command::new("zsh")
        .arg("-lc")
        .arg(command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| format!("failed to execute command: {error}"))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| "failed to capture command stdout".to_string())?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| "failed to capture command stderr".to_string())?;

    let stdout_handle = spawn_reader(stdout, "stdout");
    let stderr_handle = spawn_reader(stderr, "stderr");

    let timeout = Duration::from_secs(timeout_seconds.max(1));
    let mut timed_out = false;
    loop {
        if child
            .try_wait()
            .map_err(|error| format!("failed to wait command: {error}"))?
            .is_some()
        {
            break;
        }
        if started.elapsed() >= timeout {
            timed_out = true;
            let _ = child.kill();
            break;
        }
        thread::sleep(Duration::from_millis(40));
    }

    let status = child
        .wait()
        .map_err(|error| format!("failed to collect command output: {error}"))?;
    let stdout_buffer = stdout_handle
        .join()
        .map_err(|_| "failed to collect command stdout".to_string())??;
    let stderr_buffer = stderr_handle
        .join()
        .map_err(|_| "failed to collect command stderr".to_string())??;

    let duration_ms = started.elapsed().as_millis();
    let mut stderr = stderr_buffer.trim().to_string();
    if timed_out {
        let timeout_message = format!("command timed out after {}s", timeout_seconds.max(1));
        stderr = if stderr.is_empty() {
            timeout_message
        } else {
            format!("{stderr}\n{timeout_message}")
        };
    }

    Ok(CommandOutput {
        command: command.to_string(),
        exit_code: if timed_out {
            -124
        } else {
            status.code().unwrap_or(-1)
        },
        stdout: stdout_buffer.trim().to_string(),
        stderr,
        duration_ms,
        timed_out,
    })
}
