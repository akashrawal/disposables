# Disposables

Disposables is a library that runs your test dependencies in containers.
Once your tests finish running, the containers are automatically cleaned up.
Disposables works with both Docker and Podman, does not require Docker socket
access, and works well with user namespaces.

Disposables needs Podman or Docker CLI to work.

