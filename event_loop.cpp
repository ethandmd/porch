#include "event_loop.h"

EventLoop::EventLoop()
    : lock_(), cond_(), thread_(&EventLoop::loop, this), quit_(false)
{
    thread_.join();
}

EventLoop::~EventLoop()
{
    quit();
}

void EventLoop::loop()
{
    while (!quit_.load(std::memory_order_acquire)) {
        Task task;
        {
            std::unique_lock<std::mutex> lock(lock_);
            // https://en.cppreference.com/w/cpp/thread/condition_variable
            if (pendingTasks_.empty()) {
                cond_.wait(lock);
            }
            if (!pendingTasks_.empty()) {
                task = pendingTasks_.front();
                pendingTasks_.pop_front();
                lock.unlock();
                cond_.notify_one();
            }
        }
        if (task) {
            task();
        }
    }
}
void EventLoop::queueTask(const Task& cb)
{
    {
        std::unique_lock<std::mutex> lock(lock_);
        pendingTasks_.push_back(cb);
    }
    cond_.notify_one();
}

void EventLoop::quit() {
    quit_.store(true, std::memory_order_release);
}
