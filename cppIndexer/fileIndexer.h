//Header file for fileIndexer class
//cedelmaier

/*
	fileIndexer reads the values pushed onto the safe queue and then 
	counts the words in them.  It is initialized with everything
	except the master word count vector, which is passed in when we
	run the program.  Keeps track of it's own index, the queue, MCP,
	and it's own file

    When poisoned, exits
*/

#ifndef FILEINDEXER_H
#define FILEINDEXER_H

#include <string>
#include <iostream>
#include <fstream>
#include <sstream>
#include <map>

#include "sQueue.h"
#include "ncppfi.h"

class fileIndexer {
    private:
    	int					mIndex;
    	sQueue<cppfiData>& 	mDataq;
        controlStructure&   mMCP;

    	std::ifstream				mFile;
    public:
        fileIndexer(int index, sQueue<cppfiData>& safeQueue, controlStructure& MCP)
         	: mIndex(index), mDataq(safeQueue), mMCP(MCP) {
            #ifdef DEBUG
            std::cout << "fileIndexer[" << mIndex << "] constructed\n";
            #endif
            };
        void runFileIndexer(wvmap& mVecWordCount);
};

#endif
