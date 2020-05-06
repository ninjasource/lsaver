# L-Saver

A Windows 10 screen saver which draws random l-system strings to the screen.

MIT License

The project can be run by simply running "cargo run". However, you will have to follow the steps below to install it as a screen saver.

This is project started off as a rust port of a Javascript library written by Ehren Julien-Neitzert under an MIT license. It will diverge as I continue to refactor it to my liking. I would like to thank Ehren for publishing his source code as I found the whole concept of lsystems fascinating and he made it accessible to me.
The source code for that project can be found here:
https://github.com/ehrenjn/LSystems

A Windows screen saver is simply a .exe file that has been renamed to .scr file, copied to c:\windows\system32 and registered in the registry. You can do this manually (a .scr file can be installed by right clicking it) or run the .bat scripts that I have supplied. Note that you will have to run these scripts from an elevated command prompt (administrator privileges) because they copy lsaver.scr to the system32 folder.