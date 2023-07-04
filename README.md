# rust_image_share

Hosts a Local server with a rust Server, it is a modified server as the one from the rust book https://doc.rust-lang.org/book/ch20-00-final-project-a-web-server.html. Displays a page from the Zpevnik folder with the specified name. Users are able to change their Local page number using the arrows, sync up to the global page number. The user in the /admin domain (I think that's what it's called) can change the Global page value. The biggest problem right now is that when a new user signs up they dont get their globalPage variable, which is stored localy, updated until the admin changes the page or manually sends the update emit.

The goal is to use this in a remote enviroment where there might not be internet connection to share the chords and the text to songs. I know very usefull for the general population :D
