# Palangrotte

This is a simple daemon that monitors a series of directories containing canary files for changes. When changes are detected, it sends notifications to a specified service, logs the event, displays a message to the possible opened sessions and forces the system to shut down.
The project is intended to be used in a production environment on Windows, although it can also be used on Linux. It's designed to be run as a service.
It uses the crate `notify` for file system event monitoring.

## Configuration

The directories to be monitored are specified in the `folders.txt` file. Each line in this file should contain the absolute path to a directory that you want to monitor.

If a directory specified in `folders.txt` doesn't exist, the application will create it.
If a directory is empty, the application will populate it with canary files.
If a directory is not readable or writable, it will be skipped, and an error will be logged in `plgrt.log`.

## Usage

To use the application, you first need to configure the `folders.txt` file with the directories you want to monitor. Once you have configured the folders, you can run the application using the following command:

```bash
cargo run --bin palangrotte
```

The application will then start monitoring the specified directories for changes. When a file in one of the monitored folders is modified, a message will be printed to the console indicating which file or folder was changed. All setup events and errors will be logged to the `plgrt.log` file.

## Error Handling

The application is designed to be robust. If it encounters an issue with a specific folder (e.g., a permissions error), it will log the problem in `plgrt.log` and continue trying to monitor the other folders listed in your configuration.

However, if the application is unable to monitor *any* of the specified folders, it will consider this a critical failure. In this case, it will print an error message to the console, write a final entry to the log file, and then exit. This prevents the service from running silently without actually performing its monitoring duties.