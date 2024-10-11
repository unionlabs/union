// This is an example kqueue program which watches a directory and all paths in
// it with the same flags as those fsnotify uses. This is useful sometimes to
// test what events kqueue sends with as little abstraction as possible.
//
// Note this does *not* set up monitoring on new files as they're created.
//
// Usage:
//   cc kqueue.c -o kqueue
//   ./kqueue /path/to/dir

#include <sys/event.h>
#include <sys/time.h>
#include <dirent.h>
#include <fcntl.h>
#include <stdarg.h>
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <unistd.h>

void die(const char *fmt, ...) {
	va_list ap;
	va_start(ap, fmt);
	vfprintf(stderr, fmt, ap);
	va_end(ap);

	if (fmt[0] && fmt[strlen(fmt)-1] == ':') {
		fputc(' ', stderr);
		perror(NULL);
	}
	else
		fputc('\n', stderr);

	exit(1);
}

int main(int argc, char* argv[]) {
	if (argc < 2) {
		fprintf(stderr, "usage: %s path/to/dir\n", argv[0]);
		return 1;
	}
	char *dir = argv[1];

	int kq = kqueue();
	if (kq == -1)
		die("kqueue:");

	int fp = open(dir, O_RDONLY);
	if (fp == -1)
		die("open: %s:", dir);
	DIR *dp = fdopendir(fp);
	if (dp == NULL)
		die("fdopendir:");

	int    fds[1024]     = {fp};
	char   *names[1024]  = {dir};
	int    n_fds         = 0;
	struct dirent *ls;
	while ((ls = readdir(dp)) != NULL) {
		if (ls->d_name[0] == '.')
			continue;

		char *path = malloc(strlen(dir) + strlen(ls->d_name) + 2);
		sprintf(path, "%s/%s", dir, ls->d_name);

		int fp = open(path, O_RDONLY);
		if (fp == -1)
			die("open: %s:", path);
		fds[++n_fds] = fp;
		names[n_fds] = path;
	}

	for (int i=0; i<=n_fds; i++) {
		struct kevent changes;
		EV_SET(&changes, fds[i], EVFILT_VNODE,
				EV_ADD | EV_CLEAR | EV_ENABLE,
				NOTE_DELETE | NOTE_WRITE | NOTE_ATTRIB | NOTE_RENAME,
				0, 0);

		int n = kevent(kq, &changes, 1, NULL, 0, NULL);
		if (n == -1)
			die("register kevent changes:");
	}

	printf("Ready; press ^C to exit\n");
	for (;;) {
		struct kevent event;
		int n = kevent(kq, NULL, 0, &event, 1, NULL);
		if (n == -1)
			die("kevent:");
		if (n == 0)
			continue;

		char *ev_name = malloc(128);
		if (event.fflags & NOTE_WRITE)
			strncat(ev_name, "WRITE ", 6);
		if (event.fflags & NOTE_RENAME)
			strncat(ev_name, "RENAME ", 6);
		if (event.fflags & NOTE_ATTRIB)
			strncat(ev_name, "CHMOD ", 5);
		if (event.fflags & NOTE_DELETE) {
			strncat(ev_name, "DELETE ", 7);
			struct kevent changes;
			EV_SET(&changes, event.ident, EVFILT_VNODE,
					EV_DELETE,
					NOTE_DELETE | NOTE_WRITE | NOTE_ATTRIB | NOTE_RENAME,
					0, 0);
			int n = kevent(kq, &changes, 1, NULL, 0, NULL);
			if (n == -1)
				die("remove kevent on delete:");
		}

		char *name;
		for (int i=0; i<=n_fds; i++)
			if (fds[i] == event.ident) {
				name = names[i];
				break;
			}

		printf("%-13s %s\n", ev_name, name);
	}
	return 0;
}
