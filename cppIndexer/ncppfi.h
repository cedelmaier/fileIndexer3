//Structures and global functions
//cedelmaier

#ifndef NCONTROL_H
#define NCONTROL_H

#include <mutex>
#include <thread>
#include <condition_variable>
#include <map>
#include <atomic>

//typedefs
typedef std::multimap<int, std::string>::reverse_iterator mmri;
typedef std::vector<std::map<std::string, int>* > wvmap;

//Control structure for when the fileFinder ends.  Moved to a poison pill
//implementation.  Just keep track of nThreads
struct controlStructure {
    int nThreads;
    bool printIndexing;
};

//We can also use the poison pill implementation, which is easier and faster
struct cppfiData {
    std::string fileName    = "";
    bool killSwitch         = false;
};

//We need a way to compare the second element of our word count maps
struct mapValueCompare {
    template <typename Lhs, typename Rhs>
    bool operator()(const Lhs& lhs, const Rhs& rhs) const
    {
        return lhs.second < rhs.second;
    }
};

//We also need some functions for swapping from a map of strings->ints to 
//ints->strings
template<typename A, typename B>
std::pair<B,A> flip_pair(const std::pair<A,B> &p) {
    return std::pair<B,A>(p.second, p.first);
}

template<typename A, typename B>
std::multimap<B,A> flip_map(const std::map<A,B> &src) {
    std::multimap<B,A> dst;
    std::transform(src.begin(), src.end(), std::inserter(dst, dst.begin()), 
                   flip_pair<A,B>);
    return dst;
}

#endif
