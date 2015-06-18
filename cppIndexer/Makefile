CXX = g++
CXXFLAGS = -g -Wall -Wextra
CXXFLAGS += -I/usr/local/include
#LDFLAGS = -L/usr/lib/x86_64-linux-gnu/
#LDLIBS = -lboost_system -lboost_filesystem -lboost_regex -lboost_timer -lboost_atomic -lpthread
#THREADS = -std=c++11 -pthread
LDFLAGS = -L/usr/local/lib
LDLIBS = -lboost_system-mt -lboost_filesystem-mt -lboost_timer-mt
THREADS = -std=c++11

all: normal

normal: CXXFLAGS += -O0
normal: cppfi

release: CXXFLAGS += -O3
release: cppfi

debug: CXXFLAGS += -O0 -DDEBUG
debug: cppfi

cppfi: cppfi.o fileFinder.o fileIndexer.o
	$(CXX) cppfi.o fileFinder.o fileIndexer.o -o cppfi $(THREADS) $(LDFLAGS) $(LDLIBS)

cppfi.o: cppfi.cpp sQueue.h ncppfi.h
	$(CXX) $(CXXFLAGS) -c cppfi.cpp $(THREADS)

fileFinder.o: fileFinder.cpp fileFinder.h sQueue.h ncppfi.h
	$(CXX) $(CXXFLAGS) -c fileFinder.cpp $(THREADS)

fileIndexer.o: fileIndexer.cpp fileIndexer.h sQueue.h ncppfi.h
	$(CXX) $(CXXFLAGS) -c fileIndexer.cpp $(THREADS)

clean:
	rm -f *.o cppfi

