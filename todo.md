* add depencies (-> maybe even interface with github -- requires git to be installed)
* option to recursevly traverse directory tree to check for other projects that can be build as depencies for the project

* check for recompilation -> Make works by inspecting information about files, not their contents. 
                             Make works out dependencies between targets and their dependencies, 
			     and then looks to see whether the files exist. If they do, it asks 
			     the operating system for the time and date the file was last modified. 
			     This is the 'timestamp' for this purpose, although the term can have other meanings.

                             If a target file either does not exist, or exists and is earlier than its dependent file,
			     then Make rebuilds the target from the dependent by applying a rule.
* add option to add system default libs at linking stage: windows-libs -> "shell32.lib advapi32.lib cfgmgr32.lib comctl32.lib comdlg32.lib d2d1.lib dwrite.lib dxgi.l										ib gdi32.lib kernel32.lib msimg32.lib ole32.lib opengl32.lib shlwapi.lib user32.lib window									     scodecs.lib winspool.lib userenv.lib ws2_32.lib bcrypt.lib msvcrt.lib oleaut32.lib uuid.li										  b odbc32.lib odbccp32.lib"
