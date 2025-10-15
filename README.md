# Palangrotte

_work in progress, no submission accepted for now_


This is a simple daemon that monitors a series of directories containing canary files for modification or removal.

When such changes are detected, it sends notifications to a specified service, logs the event, displays a message to the possible opened sessions (requires libnotify-bin on Linux) and forces the system to shut down.

The project is intended to be used in a production environment on Windows, although it can also be used on Linux. It's designed to be run as a service and to work in the user-space.

It uses the crate `notify` for file system event monitoring.

## Configuration

The application is configured through the `palangrotte.toml` file. This file allows you to customize various parameters, including the location of the log file, the URL for notifications, and the names and sizes of the canary files.

The directories to be monitored are specified in an encrypted file, by default `folders.enc`. The password for decrypting this file is also set in `palangrotte.toml` via the `keyword` parameter.

You can create and encrypt the folders file using the `encrypter` utility. First, create a plain text file (e.g., `folders.txt`) with the absolute path of each directory you want to monitor on a new line. Then, use the `encrypter` to encrypt it:

```bash
cargo run --bin encrypter encrypt folders.txt folders.enc
```

- If a directory specified in `folders.enc` doesn't exist, the application will create it.
- If a directory is empty, the application will populate it with canary files based on the settings in `palangrotte.toml`.
- If a directory is not readable or writable, it will be skipped, and an error will be logged.

## Permissions Strategy for Malware Detection

To maximize the chances of detecting malware, the folders containing canary files should be as accessible as possible. This strategy makes it trivial for malicious software, which may be running with low privileges, to access and modify a canary file, thereby triggering an alert.

It is highly recommended to set open permissions on the directories you intend to monitor *before* running the application. If you specify directories that do not yet exist, you should set these permissions on the parent directory, so the newly created canary folders inherit them.

**On Windows:**

Windows has a powerful permission inheritance model. By granting full access to the "Everyone" group on a canary folder, you ensure that all files and subfolders created within it will be fully accessible. Use the `icacls` command to apply these permissions recursively:

```bash
icacls "C:\path\to\your\canary\folder" /grant Everyone:(F) /t
```
This command grants Full control `(F)` to `Everyone` and applies it to all files and subdirectories `/t`.

**On Linux:**

On Linux, you can set wide-open permissions on the folder. The application itself will ensure that the canary files it creates are world-writable (`0o666`).

```bash
chmod 777 /path/to/your/canary/folder
```

By adopting this open-permission strategy, you lower the bar for interaction, turning the canary files into an effective tripwire for unauthorized system activity.

## File Permissions

On Linux, the canary files are created with write permissions for all users. This is to ensure that the monitoring service can detect modifications made by any user. On Windows, the default file permissions are used.

## Usage

To use the application, you first need to create the encrypted `folders.enc` file and configure your `palangrotte.toml`. Once the configuration is ready, you can run the application:

```bash
cargo run --bin palangrotte
```

The application will then start monitoring the specified directories for changes. When a file in one of the monitored folders is modified or removed, a message will be printed to the console, and a notification will be sent to the configured remote service. All setup events and errors will be logged.

## Monitoring Process

The core of the application is its file monitoring capability, which is handled by the [`notify`](https://crates.io/crates/notify) crate. Here's how it works:

1.  **Initialization**: When the application starts, it reads the list of directories to monitor from your encrypted configuration file.
2.  **Baselining**: For each directory, the application establishes a baseline. If the directory is empty, it's populated with new canary files. If files already exist, the application updates their modification timestamps. This ensures that the monitor only triggers on changes that occur *after* it has started.
3.  **Recursive Watching**: Each specified directory is monitored **recursively**, meaning any changes to files within the directory or in any of its subdirectories will be detected.
4.  **Event Detection**: The application watches for specific filesystem events, primarily file modifications and removals. When one of these events is detected within a monitored directory, the alert and shutdown sequence is triggered.

### Security Note: Identifying the Source of Changes

It's important to understand that while Palangrotte can tell you *what* file was changed and *when*, it cannot determine *who* (which user or process) made the change. This is a limitation of the underlying filesystem notification APIs used by the `notify` crate.

For a comprehensive security setup, it is highly recommended to use Palangrotte in conjunction with your operating system's native auditing tools. On Linux, the **Linux Audit Daemon (`auditd`)** is the standard for this. By correlating the real-time alert from Palangrotte with the detailed logs from `auditd`, you can pinpoint the exact user, process, and command responsible for the modification.

## Testing Notifications

The project includes a simple PHP script, `index.php`, that can be used to test the remote notification functionality. This script listens for incoming POST requests, decodes the JSON payload, and logs the timestamp, remote IP address, and the name of the modified file to a text file.

To use it, place `index.php` on a web server and make sure that the directory where the output file is written exists and is writable by the user running the web server.

## Error Handling

The application is designed to be robust. If it encounters an issue with a specific folder (e.g., a permissions error), it will log the problem in `plgrt.log` and continue trying to monitor the other folders listed in your configuration.

However, if the application is unable to monitor *any* of the specified folders, it will consider this a critical failure. In this case, it will print an error message to the console, write a final entry to the log file, and then exit. This prevents the service from running silently without actually performing its monitoring duties.

## Encryption Utility

The project includes a command-line utility for encrypting and decrypting files, which can be used to protect sensitive configuration files like `folders.txt`. The tool uses ChaCha20-Poly1305 for encryption and derives a key from a user-provided password using PBKDF2.

### Usage

To use the encryption utility, run the `encrypter` binary with one of the following commands:

**To encrypt a file:**

```bash
cargo run --bin encrypter encrypt <input_file> <output_file>
```

The tool will prompt you to enter and confirm a password.

**To decrypt a file:**

```bash
cargo run --bin encrypter decrypt <input_file> <output_file>
```

The tool will prompt you for the password to decrypt the file.

## User Session Notification

Before forcing the system to shut down, the application will attempt to notify all active user sessions about the security alert. This provides a brief warning to anyone who is currently logged in.

The notification method is platform-specific:

-   **On Windows:** A message box is displayed on the desktop of each active user session using the Windows API.
-   **On Linux:** The application executes the embedded `notify_send_all.sh` script, to broadcast a desktop notification to all graphical user sessions. Credits [https://github.com/tonywalker1/notify-send-all]
