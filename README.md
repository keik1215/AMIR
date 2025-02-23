# Auto Mod Installer.Rs
## What is it?
"Auto Mod Installer.Rs"(AMIR) is a tool that allows us to install our server's mod more conveniently. It has pretty modern GUI and can show us realtime installing status.
## How to use
First of all, open the .exe file. It usually needs nothing because its framework tauri uses WebView, which is basically usable in most of OS.  
Then, there'll be three sections below its header. Left one is where to input configure file(.json)'s url.  
Middle one is where to input .minecraft path(It is usually in %APPDATA%\\.minecraft). Below it, there's a toggle button. If you turn on this button, AMIR will skip the making-launcher-profile process.  
And the other one is in which your game directory will be put. The mods will be downloaded there.  
Finishing all settings, now press the start button. Then installation will start. there are four dots below the button. They shows us progress of installation. First one checks the configure file is correct. Second one checks whether Java and appropriate Forge is installed or not. When third one lits greenly, that means the mod downloading is done. The last one lits when minecraft launcher profile making is done, in other words, all progress is done.