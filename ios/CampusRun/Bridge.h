#ifndef CAMPUS_ROUTE_BRIDGE_H
#define CAMPUS_ROUTE_BRIDGE_H
char *route_call(const char *directory, const char *operation, const char *input);
void route_free(char *value);
#endif
