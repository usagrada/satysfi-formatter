package main

import (
	"fmt"
	"io/ioutil"
	"regexp"
	"strings"
)

const grammmerUrl = "https://raw.githubusercontent.com/monaqa/tree-sitter-satysfi/master/grammar.js"

func main() {
	data := getData()
	grammer := fmtGrammer(data)
	tokens := fmtToken(data)
	// println("data: ", grammer)
	lines := strings.Split(grammer, "\n")
	tokenList := make([]string, 0)

	tokenlines := strings.Split(tokens, "\n")

	// tokenListを抽出
	for _, line := range tokenlines {
		trimLine := strings.TrimLeft(line, " ")
		if strings.HasPrefix(trimLine, "//") {
			continue
		}
		flag := regexp.MustCompile(`[A-z_]+: `).MatchString(trimLine)
		if flag {
			token := strings.Split(trimLine, ":")[0]
			tokenList = append(tokenList, token)
		}
	}
	// grammerのruleからtokenを抽出
	for _, line := range lines {
		trimLine := strings.TrimLeft(line, " ")
		if strings.HasPrefix(trimLine, "//") {
			continue
		}
		flag := regexp.MustCompile(`[A-z_]+: \(.\) =>.*`).MatchString(trimLine)
		if flag {
			token := strings.Split(trimLine, ":")[0]
			tokenList = append(tokenList, token)
		}
	}

	fmt.Println("#[allow(dead_code, non_camel_case_types)]")
	fmt.Println("#[derive(Debug, Clone, PartialEq, PartialOrd)]")
	fmt.Println("pub enum Token {")
	for _, token := range tokenList {
		fmt.Println("\t" + token + ",")
	}
	fmt.Println("\tother(String),")
	fmt.Println("}")
}

func fmtToken(data string) string {
	// "module.exports = grammar({" + data + "});"
	index := strings.Index(data, "function tokens() {")
	if index == -1 {
		panic("Error: could not find const tokens = {")
		// return data
	}
	data = data[index+len("function tokens() {"):]
	last_index := strings.Index(data, "};")
	if last_index == -1 {
		panic("Error: could not find };")
	}
	return data[:last_index+1]
}

func fmtGrammer(data string) string {
	// println("data: ", data)
	// "module.exports = grammar({" + data + "});"
	index := strings.Index(data, "module.exports = grammar({")
	if index == -1 {
		panic("Error: could not find module.exports = grammar({")
		// return data
	}
	data = data[index+len("module.exports = grammar("):]
	index = strings.Index(data, "});")
	if index == -1 {
		panic("Error: could not find });")
	}
	return data[:index+1]
}

func getData() string {
	fileName := "./scripts/grammar.v0_1_0.js"
	bytes, err := ioutil.ReadFile(fileName)
	if err != nil {
		panic(err)
	}
	body := string(bytes)
	// println("body: ", body)

	// res, err := http.Get(grammmerUrl)
	// if err != nil {
	// 	panic(err)
	// }
	// defer res.Body.Close()

	// byte_body, _ := ioutil.ReadAll(res.Body)
	// body := string(byte_body)
	// println(res.StatusCode, body)
	return body
}
