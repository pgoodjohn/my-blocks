import React from 'react';
import { Link } from '@tanstack/react-router';

interface BlockProps {
    block: any
    id: string
    raw_data: string
}

const Block: React.FC<BlockProps> = ({ block }) => {
    return (
        <div id={block.id} className="flex group/block-content">
            {block.block_type === 'page' &&
                <Link className="underline" to={`/page/${block.id}`}>{block.block_contents.contents}</Link>
            }
            {block.block_type === 'text' &&
                <p>{block.block_contents.contents}</p>
            }
        </div >
    )
}

export default Block;