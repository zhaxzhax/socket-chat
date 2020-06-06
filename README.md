# socket-chat
> A chat demo based on socket which helps me deeply understanding socket.
### why code this?
That's because I am going to build a p2p-chat application, so I code this repo to help me understanding `std::net` crate, and how to build mutithreads application in rust.
## how to use
> cargo run 

Then change directory to electron-quick-start, open three terminals. Input the command in each terminals.

> npm start

The chat application is designed for three terminal, it will fail when you shut down a terminal process.

## problem
+ Due to Rust's ownership design, my streamPool can not be owned by two threads. I am sure that shared data in streamPool all be locked by the smart pointer `Arc` and `Mutex`. But streamPool still can not be shared. And because of streamPool is not `sized` , I can't use `Mutex` to lock it. So we have to wait three person join the channel that the message can be transfer later (One thread, connect before handle connection ).

+ The threads are blocked in `stream.read(&mut buffer)`. So I choose to send meaningless message ' ` ' to make sure the connection will not be dead locked. 
## some methods may solve the problems

The first problem is so disappointed that we can't get a varible size of chat channel. The second push we to waste network bandwidth.

My suggestion is
> Use channel to share memory, not through shared memory

If I choose to use message channel to transfer `stream` between two threads. That would be no more problems about `Mutex`.

Besides, to solve the second problem, I need to use no-blocking read rather than blocking read. Or still solve it by message channel: send the message to other stream threads, tell them how to write the stream.
