function noop() {}
var is_secure = function () {
    return true;
}

function HTMLCollection() {}
HTMLCollection.prototype.item = function () {
    return new Element();
}

function Element() {
    this.innerHTML = '';
    this.childNodes = [];
    this.id = null;
}
Element.prototype.appendChild = function (node) {
    this.childNodes.push(node);
}
Element.prototype.setAttribute = noop;

function DocumentFragment() {
    this.children = [];
}

var document = {
    _nodes: new Element(),
};
document.createElement = function (name) {
    return new Element(name);
}
document.createDocumentFragment = function () {
    return new DocumentFragment();
}
document.getElementById = function (id) {
    var nodes = document._nodes.childNodes;
    for (var i = 0; i < nodes.length; i++) {
        if (nodes[i].id == id)
            return nodes[i];
    }

    var el = new Element();
    el.id = id;
    nodes.push(el);

    return el;
}
document.createTextNode = noop;
document.getElementsByTagName = function (name) {
    return new HTMLCollection();
}
document.write = noop;

var window = {};
window.location = {};
window.location.href = '';
