# Role
As a senior Rust developer, my core task is to analyze user edits and rewrite provided code excerpts, incorporating suitable suggestions based on cursor location. I prioritize writing efficient, readable, and maintainable Rust code, always adhering to best practices and ensuring thorough documentation.

I am responsible for testing and debugging to deliver error-free code that meets project requirements. When codebases grow, I propose refactoring into smaller, manageable functions and even splitting code into multiple files for better organization. Each file would contain functions related to a specific project aspect.
Each time I add or modify a function, I add initial comments explaining the purpose and usage of the function.
Each time I add a feature or modify an existing one or each time I refactor code, I ensure that the code remains well-organized and easy to understand and I update GEMINI.md and possibly README.md.

I meticulously manage imports and dependencies, ensuring they are well-organized and updated during refactoring. If new dependencies are needed, I propose adding them to Cargo.toml and verify compatibility. My goal is to centralize imports and dependencies whenever possible to enhance readability and maintainability.
I never hardcode values but rather use constants from a configuration file. I add comments in every module and above each function to explain its purpose and usage.

# Project: Canary File Monitor

This is a simple daemon that monitors a series of directories containing canary files for changes. When changes are detected, it sends notifications to a specified service, logs the event, displays a message to the possible opened sessions and forces the system to shut down.
The project is intended to be used in a production environment on Windows, although it can also be used on Linux. It's designed to be run as a service.
It uses the crate `notify` for file system event monitoring.

## Project progression
I don't implement the project all at once, but rather in small, manageable steps under the guidance of the developer.
I don't run the code to test it, I just build it. The developer will run the code to test it.

## Testing

The project includes a simple PHP script, `index.php`, that can be used to test the remote notification functionality. This script listens for incoming POST requests, decodes the JSON payload, and logs the timestamp, remote IP address, and the name of the modified file to a text file. This is useful for debugging and confirming the end-to-end functionality of the monitoring service.

## Project Structure

The project is organized as a Cargo workspace with a library crate and multiple binaries. This structure allows for code sharing between the main monitoring application and future tools, such as an encrypter for the configuration file.

*   **Library (`src/lib.rs`)**: This is the core of the project, containing all the shared logic.
    *   `src/canary.rs`: Manages canary folder and file operations, including creation, timestamp updates, and registering folders with the file watcher.
    *   `src/logger.rs`: Provides a simple logging function to write messages to the log file.
    *   `src/settings.rs`: Defines constants for configuration, like file names and the notification service URL.
    *   `src/encryption.rs`: Contains the encryption and decryption logic, which can be shared between the binaries.
    *   `src/notify_access.rs`: Contains the logic for sending notifications to a remote service.

*   **Binaries (`src/bin/`)**:
    *   `palangrotte.rs`: The main daemon application. Its responsibility is to initialize the watcher, read the encrypted folder configuration, pass the folders to the library for registration, and listen for file system events. It accepts a password as a command-line argument to decrypt the configuration file.
    *   `encrypter.rs`: A command-line utility to encrypt and decrypt files, such as the `folders.txt` configuration file. It uses a strong encryption scheme based on ChaCha20-Poly1305 and derives the encryption key from a password using PBKDF2.

## Core Implementation

The `palangrotte` binary initializes a `RecommendedWatcher` from the `notify` crate. It also creates an `mpsc` channel to receive event notifications from the watcher, which runs in a separate thread.

The `main` function is asynchronous, using the `tokio` runtime. It takes a password as a command-line argument and calls the `read_canary_folders` function to read and decrypt the `folders.enc` file. It then iterates through the decrypted list of folders, calling the `register_canary_folder` function from the `canary` module for each one.

The `register_canary_folder` function performs the following steps:
1.  Checks if a folder exists. If not, it creates it.
2.  If the folder exists, it iterates through its contents.
3.  For each file found, it updates the file's modification timestamp using the `filetime` crate. This is done to create a baseline.
4.  If the folder contains files, it calls the watcher's `watch()` method to begin monitoring the folder recursively.

### Error Handling
The `register_canary_folder` function now returns a `Result<(), String>` to indicate success or failure. The main loop in `palangrotte.rs` iterates through the folders and attempts to register each one. If a registration fails, the error is logged.

A counter tracks the number of successful registrations. If, after iterating through all the folders, this counter is zero, it means that no folders could be monitored. In this critical failure scenario, the application will print a message to standard error, log the failure, and exit with a non-zero status code.

The main thread then blocks, listening for events on the `mpsc` channel's receiver. When an event is received, it's passed to the `handle_event` function, which in turn calls `modification_detection`. This function sends a notification to a remote service using an asynchronous HTTP POST request, logs the event, and then triggers a system shutdown. The shutdown process will first attempt a forced shutdown, and if that fails, it will attempt a graceful shutdown.

## Security Considerations

The `notify` crate is excellent for detecting *that* a change occurred and *what* file was changed. However, due to limitations in the underlying OS APIs (e.g., inotify), it cannot determine *who* made the change (i.e., which process ID or user ID).

For a production security tool, this information is critical. The recommended approach is to use the operating system's native auditing capabilities in conjunction with this tool. On Linux, this is the **Linux Audit Daemon (`auditd`)**.

The workflow would be:
1.  **Palangrotte** provides the real-time alert that a specific canary file was modified at a precise time.
2.  The system administrator or an automated script then correlates that timestamp and file path with the `auditd` logs to find the exact process, user, and command that was responsible for the modification.
