//Concurrent safe queue container
//cedelmaier

//Concurrent queue container for class T.  enqueue works fine, but really do 
//need to use try_pop instead of dequeue for removing items.  This is because
//dequeue can hang and wait forever, whereas the try_pop can return and then
//check for an overall state change in the program

//Mostly taken from an online implementation I found somewhere.

#ifndef SQUEUE_H
#define SQUEUE_H

#include <thread>
#include <queue>
#include <mutex>
#include <condition_variable>

template<class T>
class sQueue {
    private:
        std::queue<T>           queue_;
        mutable std::mutex      mutex_;
        std::condition_variable condition_;

    public:
        sQueue(void) : queue_(), mutex_(), condition_() { };
        sQueue(const sQueue& otherQ) {
            this = otherQ;
        };
        ~sQueue(void) { };

        //Pushing things onto the queue
        void enqueue(T t) {
            std::lock_guard<std::mutex> lock(mutex_);
            queue_.push(t);
            condition_.notify_one();
        };

        //Pulling things off, but can hang on wait
        T dequeue(void) {
            std::unique_lock<std::mutex> lock(mutex_);
            while(queue_.empty()) {
                condition_.wait(lock);
            }
            T val = std::move(queue_.front());
            queue_.pop();
            return val;
        };

        //dequeue with a wait timeout so that we can check if something happened
        bool try_pop(T& t, std::chrono::milliseconds timeout) {
            std::unique_lock<std::mutex> lock(mutex_);
            if(!condition_.wait_for(lock, timeout, [this] {return !queue_.empty();} )) {
                return false;
            }
            t = std::move(queue_.front());
            queue_.pop();
            return true;
        };

        //Check if the queue is empty
        bool empty() {
            std::unique_lock<std::mutex> lock(mutex_);
            return queue_.empty();
        };
};

#endif