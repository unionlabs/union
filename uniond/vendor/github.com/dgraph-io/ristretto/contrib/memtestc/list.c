/*
 * Copyright 2020 Dgraph Labs, Inc. and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

// A simple C program for traversal of a linked list
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

struct Node {
	int data;
  char* buf;
	struct Node* next;
};

// This function prints contents of linked list starting from
// the given node
void printList(struct Node* n)
{
	while (n != NULL) {
		printf(" %d ", n->data);
		n = n->next;
	}
}

long long int lo = 1L << 30;
long long int hi = 16L << 30;

struct Node* newNode(int sz) {
  struct Node* n = (struct Node*)calloc(1, sizeof(struct Node));
  n->buf = calloc(sz, 1);
  for (int i = 0; i < sz; i++) {
    n->buf[i] = 0xff;
  }
  n->data = sz;
  n->next = NULL;
  return n;
}

void allocate(struct Node* n, int sz) {
  struct Node* nn = newNode(sz);
  struct Node* tmp = n->next;
  n->next = nn;
  nn->next = tmp;
}

int dealloc(struct Node* n) {
  if (n->next == NULL) {
    printf("n->next is NULL\n");
    exit(1);
  }
  struct Node* tmp = n->next;
  n->next = tmp->next;
  int sz = tmp->data;
  free(tmp->buf);
  free(tmp);
  return sz;
}

int main()
{
  struct Node* root = newNode(100);

  long long int total = 0;
  int increase = 1;
  while(1) {
    if (increase == 1) {
      int sz = (1 + rand() % 256) << 20;
      allocate(root, sz);
      if (root->next == NULL) {
        printf("root->next is NULL\n");
        exit(1);
      }
      total += sz;
      if (total > hi) {
        increase = 0;
      }
    } else {
      int sz = dealloc(root);
      total -= sz;
      if (total < lo) {
        increase = 1;
        sleep(5);
      } else {
        usleep(10);
      }
    }

    long double gb = total;
    gb /= (1 << 30);
    printf("Total size: %.2LF\n", gb);
  };

	return 0;
}

