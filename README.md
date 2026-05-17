## Tutorial 1.2
![alt text](<Screenshot 2026-05-18 001725.png>)

spawner.spawn() queues the async task, it does not run it immediately. So the next line executes first on the main thread. Only when executor.run() is called the task starts executing and prints "howdy!". Then it hits the TimerFuture await, pauses 2 seconds, then prints "done!"
