//A go implementaiton of the fileIndexer
//Following pipelining the best that I can

package main

import (
	"bufio"
	"crypto/rand"
	"encoding/base64"
	"errors"
	"flag"
	"fmt"
	"log"
	"os"
	"path/filepath"
	"runtime"
	"sort"
	"strconv"
	"strings"
	"sync"
	"time"
)

var nthreads int = 2
var pbool bool = true

var nthreadsstr = flag.String("t", "", "number of threads")
var idir = flag.String("d", "", "directory to index")
var pflag = flag.String("p", "", "print progress")

//Bundle together our strings and errors
type result struct {
	lcword string
	err    error
}

func fileFinder(done <-chan struct{}, path string) (<-chan string, <-chan error) {
	//We are going to own the filepaths and error channels
	filepaths := make(chan string)
	errc := make(chan error, 1)

	//Boot up the anonymous function that returns the filenames
	go func() {
		defer close(filepaths)

		errc <- filepath.Walk(path, func(path string, _ os.FileInfo, err error) error {
			//Error fired, return
			if err != nil {
				return err
			}

			//Check to make sure it is a text file
			if filepath.Ext(path) != ".txt" {
				return nil
			}

			//Handle the case that we've either found a path or the done signal
			select {
			case filepaths <- path:
			case <-done:
				return errors.New("fileFinder cancelled")
			}
			return nil
		})
	}()

	return filepaths, errc
}

func fileIndexer(done <-chan struct{}, filepaths <-chan string, c chan<- result) {
	//Try to create a unique name for our indexer, usually unreadable
	index, e1 := GenerateRandomString(8)
	if e1 != nil {
		log.Fatal(e1)
	}
	fmt.Printf("Indexer[%v] coming online\n", index)

	//Should receive all results from finder
	for path := range filepaths {
		//Do something with the filepath here!!!
		if pbool {
			fmt.Printf("\tIndexer[%s] indexing: %v\n", index, path)
		}
		file, ferr := os.Open(path)
		//defer file.Close()
		//This doesn't close file until end of function, not scope!

		if ferr != nil {
			log.Fatal(ferr)
		}

		scanner := bufio.NewScanner(file)

		for scanner.Scan() {
			//Split the string
			//Implement a faster version here, Regexp used to be used as
			//var alphasplit = regexp.MustCompile(`\W`)
			//rs := alphasplit.Split(strings.ToLower(scanner.Text()), -1)
			rs := strings.FieldsFunc(strings.ToLower(scanner.Text()), inverseTokenize)
			for _, anelement := range rs {
				if anelement == "" {
					continue
				}
				r := result{anelement, nil}
				select {
				case c <- r:
				case <-done:
					log.Printf("Indexer[%s] received done\n", index)
					return
				}
			}
		}

		file.Close()

		if pbool {
			fmt.Printf("\tIndexer[%s]\tDONE indexing: %v\n", index, path)
		}

		if err := scanner.Err(); err != nil {
			log.Printf("Indexer[%s]:%v scanner error: %v\n", index, path, err)
		}
	}

	//Normal shutdown
	fmt.Printf("Indexer[%s] shutting down\n", index)
}

//ssfi runs the main program, only accepts input from main
func ssfi(rootDir string) (map[string]int, error) {
	defer timeTrack(time.Now(), "ssfi")
	//Setup our done signal
	done := make(chan struct{})
	defer close(done)

	//Start returning txt files on filepaths
	filepaths, errc := fileFinder(done, rootDir)

	//Start up our indexers to catch the results
	c := make(chan result)
	var wg sync.WaitGroup
	wg.Add(nthreads)
	for i := 0; i < nthreads; i++ {
		go func() {
			fileIndexer(done, filepaths, c)
			wg.Done()
		}()
	}
	//Startup another thread to close the results channel
	go func() {
		wg.Wait()
		//log.Printf("ssfi closing results channel\n")
		close(c)
	}()

	//Collect the results in a map, should be thread safe by
	//channel construction
	//Is this where were are seeing the mojority of the loss of
	//performance compared to the C version?
	m := make(map[string]int)
	//This position is serialized, but should be super fast, no?
	for r := range c {
		if r.err != nil {
			log.Println("ssfi failed on results channel")
			return nil, r.err
		}
		//Incrment this word
		//This should be thread safe since it can only access
		//the map when it receives a message
		m[r.lcword]++
	}

	if err := <-errc; err != nil {
		log.Fatal("ssfi failed!")
		return nil, err
	}

	return m, nil
}

func main() {
	defer timeTrack(time.Now(), "ssfi_main")
	//One has to set GOMAXPROCS otherwise there is no performance
	//increase for using go threads
	runtime.GOMAXPROCS(32)
	rootDir := ""

	//Collect command line flags
	flag.Parse()

	if *idir != "" {
		rootDir = *idir
	} else {
		log.Println("Need to define a directory!")
		return
	}

	if *nthreadsstr != "" {
		nthreads, _ = strconv.Atoi(*nthreadsstr)
	}
	if *pflag != "" {
		pbool = false
	}

	m, err := ssfi(rootDir)
	if err != nil {
		log.Fatal(err)
	}

	//Use the online suggestion of how to use slices to make a map sort
	p := sortMapByValue(m)

	counter := 0
	for i := len(p) - 1; counter < 10; i-- {
		fmt.Printf("[%v]\t%v\n", p[i].Key, p[i].Value)
		counter++
	}

	log.Println("Main shutting down", err)
}

func timeTrack(start time.Time, name string) {
	elapsed := time.Since(start)
	log.Printf("%s took %s", name, elapsed)
}

//Grabbed from online to generate random strings
func GenerateRandomBytes(n int) ([]byte, error) {
	b := make([]byte, n)
	_, err := rand.Read(b)
	return b, err
}

func GenerateRandomString(s int) (string, error) {
	b, err := GenerateRandomBytes(s)
	return base64.URLEncoding.EncodeToString(b), err
}

// Sort map by value magic!
// A data structure to hold a key/value pair.
type Pair struct {
	Key   string
	Value int
}

// A slice of Pairs that implements sort.Interface to sort by Value.
type PairList []Pair

func (p PairList) Swap(i, j int)      { p[i], p[j] = p[j], p[i] }
func (p PairList) Len() int           { return len(p) }
func (p PairList) Less(i, j int) bool { return p[i].Value < p[j].Value }

// A function to turn a map into a PairList, then sort and return it.
func sortMapByValue(m map[string]int) PairList {
	p := make(PairList, len(m))
	i := 0
	for k, v := range m {
		p[i] = Pair{k, v}
		i++
	}
	sort.Sort(p)
	return p
}

//Magic here
//inverseTokenize finds all alphanumeric words and splits
//on anything else
func inverseTokenize(c rune) bool {
	switch c {
	case 'a',
		'b',
		'c',
		'd',
		'e',
		'f',
		'g',
		'h',
		'i',
		'j',
		'k',
		'l',
		'm',
		'n',
		'o',
		'p',
		'q',
		'r',
		's',
		't',
		'u',
		'v',
		'w',
		'x',
		'y',
		'z',
		'A',
		'B',
		'C',
		'D',
		'E',
		'F',
		'G',
		'H',
		'I',
		'J',
		'K',
		'L',
		'M',
		'N',
		'O',
		'P',
		'Q',
		'R',
		'S',
		'T',
		'U',
		'V',
		'W',
		'X',
		'Y',
		'Z',
		'0',
		'1',
		'2',
		'3',
		'4',
		'5',
		'6',
		'7',
		'8',
		'9',
		'_':
		return false
	}
	return true
}
