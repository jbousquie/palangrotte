<?php
$json = file_get_contents('php://input');
$ts = date('Y-m-d H:i:s');
$ip = $_SERVER['REMOTE_ADDR'];

$data = json_decode($json, true);
$file = $data["file"];

$line = "$ts _ $ip _ $file";

exec("/usr/bin/echo $line >> /home/jerome/public_html/palangrotte/canary.txt");
?>
