function updateGlobalPage(symbol='') {
    if (symbol === '+'){
        globalPage -= (-1);
    } else if (symbol === '-') {
        globalPage -= (1);
        if (globalPage < 1) {
            globalPage = 1;
        }
    } else if (!(symbol ==="boobs")){
        globalPage = Number(document.getElementById("newGlobalPage").value);
    };
    emitGlobalPage();

    localPage = globalPage;
    loadZpevnik(localPage);
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

function emitGlobalPage() {
    const dataToSend = { number: globalPage };

    fetch("/emiting-global", {
    method: "POST",
    headers: {
        "Content-Type": "application/json",
    },
    body: JSON.stringify(dataToSend),
    })
    .then((response) => response.text())

    .catch((error) => {
        console.error("Error:", error);
    });
}