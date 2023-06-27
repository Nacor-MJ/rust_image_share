function updateGlobalPage(symbol='') {
    if (symbol === '+'){
        globalPage -= (-1);
    } else if (symbol === '-') {
        globalPage -= (1);
        if (globalPage < 1) {
            globalPage = 1;
        }
    } else if (!(symbol ==="boobs")){
        globalPage = document.getElementById("newGlobalPage").value;
    };
    
    syncToGlobal();
    
    // emitGlobalPage(); !!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!!! 
}

var textBox = document.getElementById("newGlobalPage");
textBox.addEventListener("keypress", function(event) {
    if (event.key === "Enter") {
        event.preventDefault(); // Prevents the default form submission behavior
        updateGlobalPage(); // Call your function here
    }
});

function move(direction) {
    updateGlobalPage(direction);
}
function sync() {
    emitGlobalPage();
}