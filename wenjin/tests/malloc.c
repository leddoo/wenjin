#include <stddef.h>

void __wasm_call_ctors();
void _start() { __wasm_call_ctors(); }


__attribute__((import_module("host"), import_name("malloc")))
extern void* alloc(void** heap, size_t n);

__attribute__((import_module("host"), import_name("fail")))
extern void fail(const char* message);

static void* heap;

void* malloc(size_t n) {
    return alloc(&heap, n);
}

typedef struct Tree {
    int value;
    struct Tree* left;
    struct Tree* right;
} Tree;

Tree* new_tree(int value, Tree* left, Tree* right) {
    Tree* tree = malloc(sizeof(Tree));
    tree->value = value;
    tree->left = left;
    tree->right = right;
    return tree;
}

Tree* tree_insert(Tree** tree, int value) {
    Tree* at = *tree;
    if(!at) {
        *tree = new_tree(value, NULL, NULL);
        return *tree;
    }

    if(at->value == value) {
        return at;
    }

    if(value < at->value) {
        return tree_insert(&at->left, value);
    }
    else {
        return tree_insert(&at->right, value);
    }
}

void tree_hash_core(Tree* tree, int* hash) {
    if(tree) {
        tree_hash_core(tree->left, hash);
        *hash += tree->value;
        *hash *= 2;
        tree_hash_core(tree->right, hash);
    }
}

int tree_hash(Tree* tree) {
    int hash = 0;
    tree_hash_core(tree, &hash);
    return hash;
}

Tree* run() {
    Tree* tree = NULL;
    tree_insert(&tree, 1);
    tree_insert(&tree, 8);
    tree_insert(&tree, 4);
    tree_insert(&tree, 7);
    tree_insert(&tree, 3);
    tree_insert(&tree, 5);
    tree_insert(&tree, 2);
    tree_insert(&tree, 6);
    if(tree_hash(tree) != ((((((((1)*2+2)*2+3)*2+4)*2+5)*2+6)*2+7)*2+8)*2) {
        fail("invalid tree hash 1-8");
    }
    return tree;
}

