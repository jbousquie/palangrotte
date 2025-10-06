##Palangrotte

This is a simple daemon that monitors a series of directories containing canary files for changes. When changes are detected, it sends notifications to a specified service, logs the event, displays a message to the possible opened sessions and forces the system to shut down.
The project is intended to be used in a production environment on Windows, although it can also be used on Linux. It's designed to be run as a service.
It uses the crate `notify` for file system event monitoring.

The directories to be monitored are specified in a directory file, for now uncrypted.
If a directory doesn't exist, it will be created.
If a directory is empty, it will be filled with some canary files.
If a directory is not readable, it will be skipped and logged.
If a directory is not writable, it will be skipped and logged.
