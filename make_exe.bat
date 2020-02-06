@setlocal
@set PATH=%PATH%;C:\Windows\System32\downlevel;
venv\Scripts\pyinstaller.exe ^
  --hiddenimport jinja2_git ^
  --hiddenimport jinja2_time ^
  --onefile ^
  scg\scg.py
pause