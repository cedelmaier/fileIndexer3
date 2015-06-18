//fileFinder implementation
//cedelmaier

#include <iostream>
#include <thread>
#include <chrono>
#include <condition_variable>
#include <utility>

#include <boost/regex.hpp>

#include "fileIndexer.h"

//Tokenizer function
void tokenize(const std::string& str,
              std::vector<std::string>& tokens,
              const std::string& delimiters = " ") {
    std::string::size_type lastPos  = str.find_first_not_of(delimiters, 0);
    std::string::size_type pos      = str.find_first_of(delimiters, lastPos);
    while(std::string::npos != pos || std::string::npos != lastPos) {
        tokens.push_back(str.substr(lastPos, pos - lastPos));
        lastPos     = str.find_first_not_of(delimiters, pos);
        pos         = str.find_first_of(delimiters, lastPos);
    }
}

//Inverse tokenizer to find alphanumeric words
void inverseTokenize(const std::string& str,
                     std::vector<std::string>& tokens) {
    const std::string& delimiters = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789_";
    std::string::size_type lastPos  = str.find_first_of(delimiters, 0);
    std::string::size_type pos      = str.find_first_not_of(delimiters, lastPos);
    while(std::string::npos != pos || std::string::npos != lastPos) {
        tokens.push_back(str.substr(lastPos, pos - lastPos));
        lastPos     = str.find_first_of(delimiters, pos);
        pos         = str.find_first_not_of(delimiters, lastPos);
    }
}

void fileIndexer::runFileIndexer(wvmap& mVecWordCount) {
    printf("Indexer[%d] coming online\n", mIndex);
    //Setup our own internal bool variables, just need 1

    std::string myFileName = "";
    cppfiData myData;
    std::vector<std::string> tokens;

    while(1) {
        myData = mDataq.dequeue();
        if(myData.killSwitch) {
            //Poisoned, kill process
            printf("Indexer[%d] poisoned\n", mIndex);
            break;
        }

        try {
            mFile.open(myData.fileName);
            if(mMCP.printIndexing) {
                printf("\tIndexer[%d] indexing: %s\n", mIndex, myData.fileName.c_str());
            }
            
            for(std::string line; std::getline(mFile, line);) {
                tokens.clear();
                //tokenize(line, tokens, " \t[],-'/\\!\"§$%&=()<>?");
                inverseTokenize(line, tokens);
                for(auto t = tokens.begin(); t != tokens.end(); t++) {
                    std::string& currentWord = *t;
                    if(currentWord == "")
                        continue;
                    std::transform(currentWord.begin(), currentWord.end(), currentWord.begin(), ::tolower);
                    (*(mVecWordCount[mIndex]))[currentWord]++;
                }
            }

            mFile.close();
            if(mMCP.printIndexing) {
                printf("\tIndexer[%d]\tDONE indexing: %s\n", mIndex, myData.fileName.c_str());
            }
        }
        catch (std::exception& ex) {
            //Catch an exception if it's thrown, and then try again on the queue.
            std::cerr << ex.what() << std::endl;
            mFile.close();
        }
    }

    printf("Indexer[%d] shutting down\n", mIndex);

    return;
}

