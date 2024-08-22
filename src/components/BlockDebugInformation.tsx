import React from "react";

type BlockDebugInformationProps = {
    block: any;
};

const BlockDebugInformation: React.FC<BlockDebugInformationProps> = ({ block }) => {

    return (
        <div className="font-mono text-xs">
            <p>Block ID: {block.id}</p>
            <p>Block Type: {block.block_type}</p>
            <p>Block contents: {JSON.stringify(block.block_contents)}</p>
            <p>Parent ID: {block.parent_id}</p>
        </div>
    )
}

export default BlockDebugInformation;