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
package_files = [
    file for file in os.listdir(path=pkgdir)
    if os.path.isfile(os.path.join(pkgdir, file))
]
import_list = [
    "@import: {}".format(file.split(".")[0])
    for file in package_files
    if file.split(".")[-1] == "satyg" or file.split(".")[-1] == "satyh"
]


def main():
    format_files()
    with open("tests/tmp.saty", mode="w") as f:
        f.write(template.replace("{import}", "\n".join(import_list)))


def format_files():
    for file in package_files:
        print(file)
        subprocess.run(
            [
                "cargo", "run", "--release",
                "{}/{}".format(pkgdir, file), "-o", "tests/{}".format(file)
            ]
        )


if __name__ == "__main__":
    main()

