function alertHiding(element, button) {
    let target = document.getElementById(element);
    let bottone = document.getElementById(button);
    target.style.visibility = 'hidden';
    target.style.position = 'absolute';
    bottone.style.visibility = 'visible';
    bottone.style.position = 'relative';
}


// handling alerts
// alert display
function alertDisplay() {
    let target = document.getElementById('alerts');
    let bottone = document.getElementById('alert_button');

    target.style.visibility = 'hidden';
    bottone.style.visibility = 'visible';

}
// alert button
function alertButton() {
    let target = document.getElementById('alerts');
    let bottone = document.getElementById('alert_button');

    target.style.visibility = 'visible';
    bottone.style.visibility = 'hidden';
}


function modalShow(id) {
    let modal = document.getElementById(id);
    modal.style.display = 'block';
}

function modalHide(id) {
    let modal = document.getElementById(id);
    modal.style.display = 'none';
}

function copySecret() {
    var copyText = document.getElementById("secret");
    copyText.select();
    copyText.setSelectionRange(0, 99999); /*For mobile devices*/
    document.execCommand("copy");
    alert("Successfully copied TOTP secret token!");
}
