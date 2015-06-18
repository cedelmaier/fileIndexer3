//fileFinder implementation
//cedelmaier

#include <iostream>
#include <thread>
#include <chrono>

#include <boost/filesystem.hpp>

#include "fileFinder.h"

namespace bfs = boost::filesystem;

void fileFinder::runFileFinder(void) {
    //Find all files with a .txt extension in our directory
    bfs::path root(mDName.c_str());
    cppfiData myData;

    //A basic search, first making sure the directory exists
    try {
        if(!bfs::exists(root)) {
            std::cout << "Directory: " << root << " does not exist!\n";

            //Send the poison pills
            for(int i = 0; i < mMCP.nThreads; i++) {
                cppfiData doneData;
                doneData.fileName = "";
                doneData.killSwitch = true;
                #ifdef DEBUG
                std::cout << "Poisoning: " << i << " sent\n";
                #endif
                mDataq.enqueue(doneData);
            }

            return;
        }

        //Nice boost filesystem usage for recursive iteration
        if(bfs::is_directory(root)) {
            bfs::recursive_directory_iterator diter(root);
            bfs::recursive_directory_iterator diterEnd;
            while(diter != diterEnd) {
                if(bfs::is_regular_file(*diter) && diter->path().extension().string() == ".txt") {
                    //Here is the safe queue, push something onto it (enqueue)
                    myData.fileName = diter->path().string();
                    myData.killSwitch = false;
                    #ifdef DEBUG
                    std::cout << "Enqueue: " << myData.fileName << " : " << myData.killSwitch << std::endl;
                    #endif
                    mDataq.enqueue(myData);
                }
                ++diter;
            }
        }
    }
    catch(const bfs::filesystem_error& ex) {
        //If we throw an error, catch it, and then we should signal all done
        //because something bad happened
        std::cerr << ex.what() << std::endl;
    }

    //Tell everybody we are done adding to the queue
    //Send the poison pills
    for(int i = 0; i < mMCP.nThreads; i++) {
        cppfiData doneData;
        doneData.fileName = "";
        doneData.killSwitch = true;
        #ifdef DEBUG
        std::cout << "Poisoning: " << i << " sent\n";
        #endif
        mDataq.enqueue(doneData);
    }

    #ifdef DEBUG
    std::cout << "fileFinder exiting\n";
    #endif

    return;
}

