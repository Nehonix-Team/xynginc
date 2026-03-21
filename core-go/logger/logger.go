package logger

import (
	"fmt"
	"strings"

	"github.com/fatih/color"
)

func init() {
	// Force colors even when running inside a piped Node child_process
	color.NoColor = false
}

func processArrowMessage(message string, colorFn func(format string, a ...interface{}) string) string {
	trimmed := strings.TrimLeft(message, " ")
	if strings.HasPrefix(trimmed, ">") {
		pos := strings.Index(message, ">")
		if pos != -1 {
			beforeArrow := message[:pos]
			afterArrow := message[pos:]
			rest := afterArrow[1:] // Remove the ">"

			arrow := color.New(color.FgRed, color.Bold).Sprint(">")
			return fmt.Sprintf("%s%s%s", beforeArrow, arrow, colorFn(rest))
		}
	}
	return colorFn(message)
}

func Info(message string) {
	fmt.Println(processArrowMessage(message, fmt.Sprintf))
}

func Success(message string) {
	fmt.Println(processArrowMessage(message, color.GreenString))
}

func Warning(message string) {
	fmt.Println(processArrowMessage(message, color.YellowString))
}

func Error(message string) {
	// eprint
	fmt.Println(processArrowMessage(message, color.RedString))
}

func Step(message string) {
	colorFn := color.New(color.FgBlue, color.Bold).SprintfFunc()
	fmt.Println(processArrowMessage(message, colorFn))
}
