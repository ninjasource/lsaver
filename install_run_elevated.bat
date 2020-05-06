cargo build --release
copy .\target\release\lsaver.exe c:\windows\system32\lsaver.scr /Y
rundll32.exe desk.cpl,InstallScreenSaver lsaver.scr