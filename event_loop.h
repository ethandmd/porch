#include <functional>
#include <list>
#include <atomic>
#include <thread>
#include <mutex>
#include <condition_variable>

class EventLoop {
public:
    typedef std::function<void()> Task;

    EventLoop();
    ~EventLoop();

    void loop();
    void quit();

    void queueTask(const Task& cb);

private:
    std::mutex lock_;
    std::condition_variable cond_;
    std::thread thread_;
    std::list<Task> pendingTasks_;
    std::atomic<bool> quit_;
};
