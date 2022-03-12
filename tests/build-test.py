import os
import subprocess


template = """@require: stdjabook
{import}

document(|
    title = {title};
    author = {author};
    show-toc = true;
    show-title = true;
|)'<>
"""

pkgdir = "{}/.satysfi/dist/packages".format(os.environ["HOME"])
package_files = [file for file in os.listdir(
    path=pkgdir) if os.path.isfile(os.path.join(pkgdir, file))]
import_list = ["@import: {}".format(file.split(".")[0])
               for file in package_files]


def format_files():
    for file in package_files:
        print(file)
        subprocess.run(
            ["satysfi-fmt",
                "{}/{}".format(pkgdir, file), "-o", "tests/{}".format(file)]
        )


format_files()
with open("tests/tmp.saty", mode="w") as f:
    f.write(template.replace("{import}", "\n".join(import_list)))