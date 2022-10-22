import re
import string
import requests


grammmerUrl = "https://raw.githubusercontent.com/monaqa/tree-sitter-satysfi/master/grammar.js"
debug = True

def main():
    grammer: string = getGrammer()
    # debug
    # print(grammer)
    tokenList = getTokenList(grammer)
    # print(tokenList)

def getTokenList(grammer: string) -> list:
    # get token list from grammer
    tokenList = []
    lines: list(string) = grammer.splitlines()
    flag = False
    tokenFlag = False

    for line in lines:
        if line.startswith("module.exports = grammar({"):
            print("start")
            flag = True
        if flag:
            line = line.strip()
            if line.startswith("//"):
                continue
            result = re.search(r"[A-z_]+: \(.\) =>.*", line)
            if result:
                if tokenFlag:
                    print("end")
                tokenFlag = False
                token = result.group().split(":")[0].strip()
                print("token:", token)
                if token.startswith("_"):
                    tokenFlag = True
                else:
                    tokenList.append(token)

            if tokenFlag:
                result = re.search(r"token\((.*)\)", line)
                if result:
                    print(result)
                    token = result.group().split(",")[0].strip()
                    tokenList.append(token)
    return tokenList

def getGrammer() -> string:
    # fetch data from grammmerUrl
    if debug:
        with open("scripts/grammar.js", "r") as f:
            text = f.read()
    else:
        data = requests.get(grammmerUrl)
        text = data.text
    return text

if __name__ == "__main__":
    main()
