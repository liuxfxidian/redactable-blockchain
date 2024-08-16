#!/bin/bash

echo "start communication now"

gnome-terminal 

cargo 'test' '--package' 'intergration_test' '--lib' '--' 'proxy::proxy_node::test' '--exact' '--nocapture'

sleep 2

gnome-terminal 

cargo 'test' '--package' 'intergration_test' '--lib' '--' 'node::node1::node1::test' '--exact' '--nocapture'

sleep 2

gnome-terminal 

cargo 'test' '--package' 'intergration_test' '--lib' '--' 'node::node2::node2::test' '--exact' '--nocapture'

sleep 2

gnome-terminal

cargo 'test' '--package' 'intergration_test' '--lib' '--' 'node::node3::node3::test' '--exact' '--nocapture'

sleep 2

gnome-terminal

cargo 'test' '--package' 'intergration_test' '--lib' '--' 'node::node4::node4::test' '--exact' '--nocapture'







