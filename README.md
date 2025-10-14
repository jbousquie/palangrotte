# Palangrotte

_work in progress, no submission accepted for now_

##TODO :
- user session notification

This is a simple daemon that monitors a series of directories containing canary files for changes.
When changes are detected, it sends notifications to a specified service, logs the event, displays a message to the possible opened sessions and forces the system to shut down.
The project is intended to be used in a production environment on Windows, although it can also be used on Linux. It's designed to be run as a service and to work in the user-space.
It uses the crate `notify` for file system event monitoring.

## Configuration

The directories to be monitored are specified in an encrypted file named `folders.enc`. You can create and encrypt this file using the `encrypter` utility.

First, create a plain text file (e.g., `folders.txt`) with the absolute path of each directory you want to monitor on a new line. Then, use the `encrypter` to encrypt it:

```bash
cargo run --bin encrypter encrypt folders.txt folders.enc
```

If a directory specified in `folders.enc` doesn't exist, the application will create it.
If a directory is empty, the application will populate it with canary files.
If a directory is not readable or writable, it will be skipped, and an error will be logged in `plgrt.log`.

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

To use the application, you first need to create the encrypted `folders.enc` file. Once you have the encrypted configuration file, you can run the application using the following command, providing the password as a command-line argument:

```bash
cargo run --bin palangrotte <password>
```

The application will then start monitoring the specified directories for changes. When a file in one of the monitored folders is modified, a message will be printed to the console, and a notification will be sent to the configured remote service. All setup events and errors will be logged to the `plgrt.log` file.

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
