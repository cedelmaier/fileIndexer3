//Super simple file indexer
//cedelmaier

#include <iostream>
#include <thread>
#include <vector>
#include <string>
#include <chrono>
#include <map>
#include <algorithm>

//For profiling, want boost auto_cpu_timer
#include <boost/timer/timer.hpp>

//Need all of our custom classes, safeQueue, controlStructure, etc
#include "sQueue.h"
#include "fileFinder.h"
#include "fileIndexer.h"
#include "ncppfi.h"

void usage(void) {
    std::cout << "Usage: ./cppfi -t <n_threads> -p <printIndexing> <directory>\n";
}

int main(int argc, char *argv[]) {
    //Immediately start the timer which will be called on exit
    boost::timer::auto_cpu_timer t;
    //Parse any command line flags we want, default 2 threads
    std::string dirName = "";
    int nThreads = 2;
    int mostUsedWords = 10;
    bool printIndexing = true;

    if(argc > 2) {
        //Have thread flag and directory
        if(std::string(argv[1]) == "-t") {
            dirName = argv[3];
            nThreads = atoi(argv[2]);
        }
        else {
            usage();
            return 0;
        }
    }
    else if(argc == 2) {
        dirName = argv[1];
        nThreads = 2;
    }
    else {
        usage();
        return 0;
    }

    //First step, setup our thread controls and variables
    //wvmap is typedef std::vector<std::map<std::string, int>* >
    //start profiling
    auto beginThreads = std::chrono::high_resolution_clock::now();
    sQueue<cppfiData>   squeue;
    controlStructure    MCP; //TRON
    wvmap               tidWordCount;

    tidWordCount.reserve(nThreads);
    MCP.nThreads = nThreads;
    MCP.printIndexing = printIndexing;

    //Create the maps on the heap for each individual thread
    for(int j = 0; j < nThreads; j++) {
        tidWordCount[j] = new std::map<std::string, int>;
    }

    //Get the file finder up and running with the squeue
    fileFinder fFileFinder(dirName, squeue, MCP);
    std::thread t1(&fileFinder::runFileFinder, fFileFinder);
    t1.detach();

    //Boot up N threads of indexers, they need to know about their place in the master map
    //Hand them the entire vector of map pointers, although they are only allowed to access their
    //own.  Enforced by hand via their index variable, in this case, i.
    std::vector<std::thread> workerThreads;
    std::vector<fileIndexer*> indexerObjects;
    for(int i = 0; i < nThreads; ++i) {
        indexerObjects.push_back(new fileIndexer(i, squeue, MCP));
        workerThreads.push_back(std::thread(&fileIndexer::runFileIndexer, indexerObjects[i], std::ref(tidWordCount)));
    }

    /*
    Here is all the work happens.  We should have one finder up and running that
    is populating the squeue, and nThreads indexers which are each keeping track
    of their own word counts.  Main should just block on the next statement.
    */

    //Wait for all the worker threads to finish
    for(auto& t : workerThreads) {
        t.join();
    }
    //t1.join();
    auto endThreads = std::chrono::high_resolution_clock::now();
    std::cout << "Thread work: " << 
        std::chrono::duration_cast<std::chrono::milliseconds>(endThreads-beginThreads).count() << "ms" << std::endl;

    //Merge the maps together into a final map from harvesting!
    //Basically the same as a reduce operation
    auto beginReduce = std::chrono::high_resolution_clock::now();
    std::map<std::string, int> finalMap;
    for(int i = 0; i < nThreads; i++) {
        std::map<std::string, int>& wMap = *(tidWordCount[i]);
        for(auto& kv : wMap) {
            finalMap[kv.first] += kv.second;
        }
    }
    //Now, swap the order!
    std::multimap<int, std::string> invertedFinalMap = flip_map(finalMap);
    int counter = 0;
    //Explicitly search backwards, the highest values with be at the end.  Basically
    //exploit how the map is sorted already.  We could throw in a vector to sort,
    //but this is already available.  remember we typedefed mmri
    for(mmri it = invertedFinalMap.rbegin(); it != invertedFinalMap.rend(); it++) {
        std::cout << "[" << it->second << "]" << "\t" << it->first << std::endl;
        counter++;
        if(counter >= mostUsedWords)  //However many words you want
            break;
    }
    auto endReduce = std::chrono::high_resolution_clock::now();
    std::cout << "Reduce work: " << 
        std::chrono::duration_cast<std::chrono::milliseconds>(endReduce-beginReduce).count() << "ms" << std::endl;

    //Destroy the indexers
    for(auto& indexer : indexerObjects) {
        delete indexer;
    }
    //Destroy the maps
    for(int j = 0; j < nThreads; j++) {
        delete tidWordCount[j];
    }

    return EXIT_SUCCESS;
}

