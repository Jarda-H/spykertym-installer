import os
import json

# NUTNO UPRAVIT PODLE VLASTNIHO UMISTENI HRY!
CESTA_KE_HRE = "C:\\Program Files (x86)\\Steam\\steamapps\\common\\[HRA]"
JMENO_JSON_SOUBORU = "manual.json"
XDELTA_EXE = "xdelta3-3.0.11-x86_64.exe"


def md5(path):
    return os.popen(f'certutil -hashfile "{path}" MD5').read().split("\n")[1]


# check if json file exists
if not os.path.exists(JMENO_JSON_SOUBORU):
    print(f"Soubor s patchem {JMENO_JSON_SOUBORU} neexistuje")
    exit()

# load json file
with open(JMENO_JSON_SOUBORU, "r") as f:
    if not f.read():
        print("Soubor s patchem je prazdny")
        exit()
    f.seek(0)
    data = json.load(f)
    if data["error"]:
        print("Chyba v souboru s patchem")
        exit()
    patch = data["patch"]

    for file in patch["files"]:
        file_path = os.path.join(CESTA_KE_HRE, file["path"].lstrip("\\"))
        old_md5 = file["old"]
        # check if the file file is ok
        current_md5 = md5(file_path)
        if current_md5 != old_md5:
            print(
                f"MD5 kontrola souboru {file["path"]} selhala. Máte originální verzi hry? {current_md5} != {old_md5}"
            )
            exit()

    for file in patch["files"]:
        file_path = os.path.join(CESTA_KE_HRE, file["path"].lstrip("\\"))
        patch_file = file["patch"] + ".patch"
        # check if the patch file exists
        if not os.path.exists(patch_file):
            print(
                f"Soubor s patchem {patch_file} neexistuje. Rozbalili jste zip s patch soubory do aktuální složky?"
            )
            exit()
        new_file = os.path.join(CESTA_KE_HRE, file["path"].lstrip("\\") + ".new")
        # patch the file
        patch = os.system(
            f'{XDELTA_EXE} -d -s "{file_path}" "{patch_file}" "{new_file}"'
        )
        # check if the file was patched
        if patch != 0:
            print(f"Chyba pri uprave souboru {file["path"]}")
            exit()
        # rename the old file to .backup
        os.rename(file_path, file_path + ".backup")
        # rename the new file to the original name
        os.rename(new_file, file_path)

        # check if the file was patched correctly
        if md5(file_path) != file["new"]:
            print(f"MD5 kontrola souboru {file["path"]} selhala")
            exit()
        f.close()
        print(f"Soubor {file["path"]} byl upraven")
print("Vsechny soubory byly upraveny")
