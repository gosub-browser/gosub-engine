#ifndef GOSUB_API_NODES_H
#define GOSUB_API_NODES_H

#include <stdbool.h>
#include <stddef.h> // for NULL (which is basically just 0... but more clear)
#include <stdint.h>
#include <stdlib.h>

#include "nodes/text.h"

struct node_t *render_tree_node_init();
void render_tree_node_free(struct node_t **node);

enum node_type_e { NODE_TYPE_ROOT = 0u, NODE_TYPE_TEXT };

struct node_t {
  enum node_type_e type;
  union data {
    bool root;               // NODE_TYPE_ROOT
    struct node_text_t text; // NODE_TYPE_TEXT
  } data;
};

struct node_t *render_tree_node_init();
void render_tree_node_free_data(struct node_t *node);
void render_tree_node_free(struct node_t **node);

#endif
