#include <pthread.h>
#include <stdio.h>
#include <errno.h>
#include <string.h>
#include <pty.h>
#include <unistd.h>
#include <stdint.h>
#include <dirent.h>
#include <stdlib.h>

const char* stimuli = "test\n\x04";

int cvt(int status) {
    if(status == -1){
        fprintf(stderr, "ERR: %d, %s\n", errno, strerror(errno));
    }
    return status;
}

void *writer(void* args) {
    int fd = (int64_t) args;
    cvt(write(fd, stimuli, strlen(stimuli)));
    printf("writer done, wrote: %zu\n", strlen(stimuli));
}

void *reader(void* args) {
    int fd = (int64_t) args;
    char buf[100] = {0};
    int len = 0;
    for(;;) {
        len = cvt(read(fd, buf, 100));
        if(len == 0 || len == -1) break;
        const char *p = buf;
        printf("CHILD: ");
        for(int i=0; i<len; i++) {
            printf("%c",*p);
            if(*p == '\n') {
                printf("CHILD: ");
            }
            p++;
        }
    }
    printf("reader done\n");
}

int inout_spawn(int input, int output) {
    pthread_t t1, t2;
    pthread_create(&t1, NULL, &reader, (int *)(int64_t)output);
    pthread_create(&t2, NULL, &writer, (int *)(int64_t)input);

    pthread_join(t1, NULL);
    pthread_join(t2, NULL);

}

int printfds() {
    DIR *dir = opendir("/proc/self/fd");
    struct dirent *entry;
    char *canon_name;
    while((entry = readdir(dir))) {
        if(entry->d_name[0] == '.'){
            continue;
        }
        char buf[100];
        sprintf(buf, "%s/%s", "/proc/self/fd", entry->d_name);
        printf("fd %s: ", entry->d_name);
        if(entry->d_type == DT_LNK) {
            canon_name = realpath(buf, NULL);
            printf("%s", canon_name);
            free(canon_name);
            canon_name = NULL;
        }
        printf("\n");
    }
}

char * const argv[] = {"cat", NULL};

int main() {
    int pts, ptm, ptm2;
    cvt(openpty(&ptm, &pts, NULL, NULL, NULL));

    printf("opentyp slave %d, master %d\n", pts, ptm);

    int pid = fork();
    switch(pid) {
    default:
        //parent
        cvt(close(pts));
        ptm2 = cvt(dup(ptm));
        printf("PID %d created, master1 %d, master2 %d\n", pid, ptm, ptm2);
        printfds();
        inout_spawn(ptm, ptm2);
    case 0:
        //child
        cvt(close(STDIN_FILENO));
        cvt(close(STDOUT_FILENO));
        cvt(close(STDERR_FILENO));
        cvt(dup2(pts, STDIN_FILENO));
        cvt(dup2(pts, STDOUT_FILENO));
        cvt(dup2(pts, STDERR_FILENO));
        cvt(close(pts));
        cvt(close(ptm));

        sleep(1);
        printfds();

        setsid();

        cvt(execv("/bin/cat", argv));
    }
}
