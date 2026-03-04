/// Spawns a command using a double-fork pattern to avoid zombie processes.
///
/// This is necessary in GTK applications because GLib installs its own SIGCHLD handler
/// that interferes with normal child process management. The double-fork ensures the
/// spawned process is completely detached and reparented to init.
pub fn spawn_detached(command: &str) {
  unsafe {
    let pid = libc::fork();
    if pid == 0 {
      // First child
      libc::setsid(); // Create new session

      // Fork again
      let pid2 = libc::fork();
      if pid2 == 0 {
        // Second child - this one actually runs the command
        let cmd_cstr = std::ffi::CString::new(command.as_bytes()).unwrap();
        libc::execl(
          b"/bin/sh\0".as_ptr() as *const i8,
          b"sh\0".as_ptr() as *const i8,
          b"-c\0".as_ptr() as *const i8,
          cmd_cstr.as_ptr(),
          std::ptr::null() as *const i8
        );
        libc::_exit(1);
      }
      // First child exits immediately
      libc::_exit(0);
    } else if pid > 0 {
      // Parent waits for first child to avoid zombies
      let mut status: i32 = 0;
      libc::waitpid(pid, &mut status, 0);
    }
  }
}
