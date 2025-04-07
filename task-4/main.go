package main

import (
	"fmt"
	"math"
	"os"
	"sort"
	"strings"

	"github.com/olekukonko/tablewriter"
)

func main() {
	indexBytes, _ := os.ReadFile("../artifacts/index.txt")
	index := strings.Split(strings.TrimSpace(string(indexBytes)), "\n")
	documentNames := make([]string, len(index))

	for i, line := range index {
		documentNames[i] = strings.Split(line, " - ")[0]
	}

	invertedIndexBytes, _ := os.ReadFile("../artifacts/inverted-index.txt")
	invertedIndex := strings.Split(strings.TrimSpace(string(invertedIndexBytes)), "\n")

	println("Start processing tf")
	tf := calculateTf(invertedIndex, documentNames)
	save("tf", tf, documentNames)
	println("Processed tf")

	println("Start processing idf")
	idf := calculateIdf(invertedIndex, len(documentNames))

	idfTableFormatted := make(map[string][]float64)

	for k, v := range idf {
		idfTableFormatted[k] = []float64{v}
	}

	save("idf", idfTableFormatted, []string{"idf"})
	println("Processed idf")

	println("Start processing tf-idf")
	tfIdf := calculateTfIdf(tf, idf)
	save("tf-idf", tfIdf, documentNames)
	println("Processed tf-idf")
}

func calculateTf(invertedIndex []string, documentNames []string) map[string][]float64 {
	tf := make(map[string][]float64)

	for _, line := range invertedIndex {
		token := strings.Split(line, " - ")[0]
		tf[token] = make([]float64, len(documentNames))
	}

	for i, documentName := range documentNames {
		documentBytes, _ := os.ReadFile(fmt.Sprintf("../artifacts/tokens/%s.txt", documentName))
		document := strings.Split(strings.TrimSpace(string(documentBytes)), "\r\n")
		occurrences := make(map[string]int, len(document))

		for _, line := range document {
			if _, ok := occurrences[line]; ok {
				occurrences[line]++
			} else {
				occurrences[line] = 1
			}
		}

		for token, occurrence := range occurrences {
			tf[token][i] = float64(occurrence) / float64(len(document))
		}
	}

	return tf
}

func calculateIdf(invertedIndex []string, documentsCount int) map[string]float64 {
	idf := make(map[string]float64)

	for _, line := range invertedIndex {
		invertedIndexEntry := strings.Split(line, " - ")
		token := invertedIndexEntry[0]
		occurrences := len(strings.Split(invertedIndexEntry[1], ", "))
		idf[token] = math.Log(float64(documentsCount) / float64(occurrences))
	}

	return idf
}

func calculateTfIdf(tf map[string][]float64, idf map[string]float64) map[string][]float64 {
	tfIdf := make(map[string][]float64)

	for token, frequences := range tf {
		tfIdf[token] = make([]float64, len(frequences))
		for i, frequence := range frequences {
			tfIdf[token][i] = frequence * idf[token]
		}
	}

	return tfIdf
}

func save(metricName string, metricData map[string][]float64, headers []string) {
	metricFileName := fmt.Sprintf("../artifacts/%s.md", metricName)
	metricFile, _ := os.OpenFile(metricFileName, os.O_APPEND|os.O_CREATE|os.O_WRONLY, 0666)
	defer metricFile.Close()

	table := tablewriter.NewWriter(metricFile)
	table.SetHeader(append([]string{"Token"}, headers...))
	table.SetBorders(tablewriter.Border{Left: true, Top: false, Right: true, Bottom: false})
	table.SetCenterSeparator("|")

	sortedTokens := make([]string, 0, len(metricData))

	for token := range metricData {
		sortedTokens = append(sortedTokens, token)
	}

	sort.Strings(sortedTokens)

	data := make([][]string, len(metricData))

	for index, token := range sortedTokens {
		data[index] = make([]string, len(metricData[token])+1)
		data[index][0] = token

		for i := range metricData[token] {
			data[index][i+1] = fmt.Sprintf("%.5f", metricData[token][i])
		}
	}

	table.AppendBulk(data)
	table.Render()
}
