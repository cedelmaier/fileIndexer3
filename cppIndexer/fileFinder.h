//Header file for fileFinder class
//cedelmaier

/*
	fileFinder is a class that recursively searches the directory given to it
	by the constructor.  If the directory isn't valid, it returns, notifying
	everything that it has quit.  It then pushes the full pathname onto the
	safe queue.  At exit, it notifies everything that it has finished, changing
	to a drain state on the queue.

    Currently, this is done as a poison pill
*/

#ifndef FILEFINDER_H
#define FILEFINDER_H

#include <string>

#include "sQueue.h"
#include "ncppfi.h"

class fileFinder {
    private:
        std::string 		mDName;
        sQueue<cppfiData>& 	mDataq;
        controlStructure& 	mMCP;

    public:
        fileFinder(const std::string &dirName, sQueue<cppfiData>& safeQueue, controlStructure& MCP) 
        	: mDName(dirName), mDataq(safeQueue), mMCP(MCP) {
            #ifdef DEBUG
            std::cout << "fileFinder constructed\n";
            #endif
        };
        void runFileFinder(void);
};

#endif
