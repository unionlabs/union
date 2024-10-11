/*
 * poll_posix: poll compatibility wrapper for POSIX systems
 * Copyright © 2013 RealVNC Ltd.
 *
 * This library is free software; you can redistribute it and/or
 * modify it under the terms of the GNU Lesser General Public
 * License as published by the Free Software Foundation; either
 * version 2.1 of the License, or (at your option) any later version.
 *
 * This library is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the GNU
 * Lesser General Public License for more details.
 *
 * You should have received a copy of the GNU Lesser General Public
 * License along with this library; if not, write to the Free Software
 * Foundation, Inc., 51 Franklin Street, Fifth Floor, Boston, MA 02110-1301 USA
 *
 */

#include <config.h>

#include <unistd.h>
#include <fcntl.h>
#include <errno.h>
#include <stdlib.h>

#include "libusbi.h"

int usbi_pipe(int pipefd[2])
{
	int ret = pipe(pipefd);
	if (ret != 0) {
		return ret;
	}
	ret = fcntl(pipefd[1], F_GETFL);
	if (ret == -1) {
		usbi_dbg("Failed to get pipe fd flags: %d", errno);
		goto err_close_pipe;
	}
	ret = fcntl(pipefd[1], F_SETFL, ret | O_NONBLOCK);
	if (ret != 0) {
		usbi_dbg("Failed to set non-blocking on new pipe: %d", errno);
		goto err_close_pipe;
	}

	return 0;

err_close_pipe:
	usbi_close(pipefd[0]);
	usbi_close(pipefd[1]);
	return ret;
}
