# Security

## What this program can do

It reads foreground window changes system wide, reads the executable path and title
of the window in front, moves keyboard focus between windows, and writes two files
under `%APPDATA%\wsmf`. It can change one system setting, the foreground lock
timeout, and one registry value under `HKCU\...\CurrentVersion\Run` if you ask it to
start with Windows.

It does not open sockets, read files it did not write, or install any hook other
than the `WINEVENT_OUTOFCONTEXT` foreground event hook. There is deliberately no
keyboard hook.

## Reporting something

Email the address on the GitHub profile, or open a private security advisory on the
repository. Please do not open a public issue for anything that would let someone
else act on it before it is fixed.

Useful to include: what an attacker would gain, whether it needs local access, and
the version from `wsmf.exe`'s file properties.

## Releases

Release binaries are built by the GitHub Actions workflow in this repository, from
the tag they claim, and the SHA-256 is published beside the executable. If a build
you have does not match, do not run it.
