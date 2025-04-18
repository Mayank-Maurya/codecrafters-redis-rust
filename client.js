const net = require('net');

const client = new net.Socket();
const PORT = 6380;
const HOST = 'localhost';

client.connect(PORT, HOST, () => {
    console.log('Connected to TCP server');

    setInterval(() => {
        const message = '*3\r\n$6\r\nCONFIG\r\n$3\r\nGET\r\n$3\r\ndir\r\n';
        console.log('Sending:', message.trim());
        client.write(message);
        return;
    }, 3000);

    // setInterval(() => {
    //     const message = '*2\r\n$4\r\nGET\r\n$3\r\nfoo\r\n';
    //     console.log('Sending:', message.trim());
    //     client.write(message);
    //     return;
    // }, 5000);
    
});

client.on('data', (data) => {
    console.log('Received raw bytes:', data); // Logs the raw Buffer
    console.log('Received as hex:', data.toString('hex')); // Logs the data in hexadecimal format
    console.log('Received as array:', [...data]); // Logs the data as an array of byte values
    const result = String.fromCharCode(...data);
    console.log(result);
    client.end();
});

client.on('close', () => {
    console.log('Disconnected from server');
});

client.on('error', (error) => {
    console.error('TCP Socket error:', error);
});
