## Tutorial 1.2
![alt text](<Screenshot 2026-05-18 001725.png>)

spawner.spawn() queues the async task, it does not run it immediately. So the next line executes first on the main thread. Only when executor.run() is called the task starts executing and prints "howdy!". Then it hits the TimerFuture await, pauses 2 seconds, then prints "done!"

## Tutorial 1.3
![alt text](<Screenshot 2026-05-18 002635.png>)
all "howdy" prints first before any "done", this is because all three tasks ran concurrently and each one hits the TimerFuture await and pauses which allows the next tasks to start

removing the drop(spawner) makes the program hang forever. Without drop(spawner), the executor's ready_queue.recv() keeps waiting indefinetly for new tasks taht will never come, so the program hangs forever
