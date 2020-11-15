#undef ESP_SOCKET
#define ESP_SOCKET_SKIP
#include <sys/socket.h>
#include <netdb.h>
#include <stdio.h>

// TODO: Move to Rust
const char *gai_strerror(int ecode) {
    return "(detailed message not available)";
}

// TODO: Move to Rust
time_t timegm(struct tm *timp) {
    return 0;
}

// TODO: Move to Rust
int pthread_atfork(void (*prepare)(void), void (*parent)(void), void (*child)(void)) {
    return 1; // TODO: Not supported
}

// TODO: Move to Rust
char *getcwd(char *buf, size_t size) {
    if (size < 2) {
        return NULL;
    }

    buf[0] = '/';
    buf[1] = '\0';
    return buf;
}

//
// TODO: Decide what to do with these 
// (a) patch libc to link with __link_name = lwip_* for espressif 
// (b) convince espressif not to do these tricks 
// (c) declare in Rust
//

int accept(int s,struct sockaddr *addr,socklen_t *addrlen)
{ return lwip_accept(s,addr,addrlen); }
int bind(int s,const struct sockaddr *name, socklen_t namelen)
{ return lwip_bind(s,name,namelen); }
int shutdown(int s,int how)
{ return lwip_shutdown(s,how); }
int getpeername(int s,struct sockaddr *name,socklen_t *namelen)
{ return lwip_getpeername(s,name,namelen); }
int getsockname(int s,struct sockaddr *name,socklen_t *namelen)
{ return lwip_getsockname(s,name,namelen); }
int setsockopt(int s,int level,int optname,const void *opval,socklen_t optlen)
{ return lwip_setsockopt(s,level,optname,opval,optlen); }
int getsockopt(int s,int level,int optname,void *opval,socklen_t *optlen)
{ return lwip_getsockopt(s,level,optname,opval,optlen); }
int closesocket(int s)
{ return lwip_close(s); }
int connect(int s,const struct sockaddr *name,socklen_t namelen)
{ return lwip_connect(s,name,namelen); }
int listen(int s,int backlog)
{ return lwip_listen(s,backlog); }
ssize_t recvmsg(int sockfd, struct msghdr *msg, int flags)
{ return lwip_recvmsg(sockfd, msg, flags); } 
ssize_t recv(int s,void *mem,size_t len,int flags)
{ return lwip_recv(s,mem,len,flags); }
ssize_t recvfrom(int s,void *mem,size_t len,int flags,struct sockaddr *from,socklen_t *fromlen)
{ return lwip_recvfrom(s,mem,len,flags,from,fromlen); }
ssize_t send(int s,const void *dataptr,size_t size,int flags)
{ return lwip_send(s,dataptr,size,flags); }
ssize_t sendmsg(int s,const struct msghdr *message,int flags)
{ return lwip_sendmsg(s,message,flags); }
ssize_t sendto(int s,const void *dataptr,size_t size,int flags,const struct sockaddr *to,socklen_t tolen)
{ return lwip_sendto(s,dataptr,size,flags,to,tolen); }
int socket(int domain,int type,int protocol)
{ return lwip_socket(domain,type,protocol); }
#ifndef ESP_HAS_SELECT
int select(int maxfdp1,fd_set *readset,fd_set *writeset,fd_set *exceptset,struct timeval *timeout)
{ return lwip_select(maxfdp1,readset,writeset,exceptset,timeout); }
#endif /* ESP_HAS_SELECT */
int ioctlsocket(int s,long cmd,void *argp)
{ return lwip_ioctl(s,cmd,argp); }

#if LWIP_POSIX_SOCKETS_IO_NAMES
ssize_t read(int s,void *mem,size_t len)
{ return lwip_read(s,mem,len); }
ssize_t write(int s,const void *dataptr,size_t len)
{ return lwip_write(s,dataptr,len); }
ssize_t writev(int s,const struct iovec *iov,int iovcnt)
{ return lwip_writev(s,iov,iovcnt); }
int close(int s)
{ return lwip_close(s); }
int fcntl(int s,int cmd,int val)
{ return lwip_fcntl(s,cmd,val); }
int ioctl(int s,long cmd,void *argp)
{ return lwip_ioctl(s,cmd,argp); }
#endif /* LWIP_POSIX_SOCKETS_IO_NAMES */

int gethostbyname_r(const char *name, struct hostent *ret, char *buf, size_t buflen, struct hostent **result, int *h_errnop)
{ return lwip_gethostbyname_r(name, ret, buf, buflen, result, h_errnop); }
struct hostent *gethostbyname(const char *name)
{ return lwip_gethostbyname(name); }
void freeaddrinfo(struct addrinfo *ai)
{ lwip_freeaddrinfo(ai); }
int getaddrinfo(const char *nodename, const char *servname, const struct addrinfo *hints, struct addrinfo **res)
{ return lwip_getaddrinfo(nodename, servname, hints, res); }
