@setlocal
@set PATH=%PATH%;C:\Windows\System32\downlevel;
venv\Scripts\pyinstaller.exe ^
  --onefile ^
  scg\scg.py
pause